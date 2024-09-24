use crate::events::Event;
use pin_project::pin_project;
use std::convert::Infallible;
use std::future::Future;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{Mutex, OwnedMutexGuard};
use jdwp_types::SuspendPolicy;

pub trait EventHandler: Clone + Send + Sized + 'static {
    type Err;
    type Future: Future<Output = Result<(), Self::Err>> + Send;

    fn handle_event(self, policy: SuspendPolicy, event: Event) -> Self::Future;
}

impl<F, Fut, Err> EventHandler for F
where
    F: FnOnce(SuspendPolicy, Event) -> Fut,
    F: Clone + Send + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
{
    type Err = Err;
    type Future = Fut;

    fn handle_event(self, policy: SuspendPolicy, event: Event) -> Self::Future {
        self(policy, event)
    }
}

type OwnedEventHandlerFn<E> =
    dyn Fn(SuspendPolicy, Event) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>> + Send + Sync;

#[must_use]
pub struct OwnedEventHandler<E = Infallible> {
    func: Arc<Mutex<OwnedEventHandlerFn<E>>>,
}

impl<E> Clone for OwnedEventHandler<E> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

impl<E> OwnedEventHandler<E> {
    pub(crate) fn new<F>(func: F) -> Self
    where
        F : EventHandler<Err=E> + Send + Sync + 'static,
    {
        let func = Arc::new(Mutex::new(move |policy: SuspendPolicy, event: Event| {
            let cloned = func.clone();
            let future = cloned.handle_event(policy, event);
            let boxed = Box::new(future) as Box<dyn Future<Output = Result<(), E>> + Send>;
            boxed.into()
        }));
        Self { func }
    }
}

impl<E: 'static> EventHandler for OwnedEventHandler<E> {
    type Err = E;
    type Future = HandleEvent<Self::Err>;

    fn handle_event<'a>(self, policy: SuspendPolicy, event: Event) -> Self::Future {
        HandleEvent::new(self, policy, event)
    }
}

#[pin_project]
pub struct HandleEvent<E> {
    #[pin]
    owned: OwnedEventHandler<E>,
    #[pin]
    state: HandleEventState<E>,
    event: Option<(SuspendPolicy, Event)>,
}

impl<E> HandleEvent<E> {
    fn new(handler: OwnedEventHandler<E>, suspend_policy: SuspendPolicy, event: Event) -> HandleEvent<E> {
        Self {
            owned: handler,
            state: HandleEventState::Init,
            event: Some((suspend_policy, event)),
        }
    }
}

enum HandleEventState<E> {
    Init,
    MutexGuardFuture(Pin<Box<dyn Future<Output = OwnedMutexGuard<OwnedEventHandlerFn<E>>> + Send>>),
    OutputFuture(Pin<Box<dyn Future<Output = Result<(), E>> + Send>>),
    Done,
}

impl<E: 'static> Future for HandleEvent<E> {
    type Output = Result<(), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut me = self.project();
        loop {
            let mut state = me.state.as_mut();
            match &mut *state {
                HandleEventState::Init => {
                    let future = me.owned.func.clone().lock_owned();
                    *state = HandleEventState::MutexGuardFuture(Box::pin(future));
                }
                HandleEventState::MutexGuardFuture(guard_future) => {
                    match guard_future.as_mut().poll(cx) {
                        Poll::Ready(item) => {
                            let (policy, event) = me.event.take().unwrap();
                            let future = item(policy, event);
                            *state = HandleEventState::OutputFuture(future)
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                HandleEventState::OutputFuture(output_fut) => {
                    return match output_fut.as_mut().poll(cx) {
                        Poll::Ready(ready) => {
                            *state = HandleEventState::Done;
                            Poll::Ready(ready)
                        }
                        Poll::Pending => Poll::Pending,
                    }
                }
                HandleEventState::Done => {
                    panic!("polled after complete")
                }
            };
        }
    }
}

pub fn handle_event<F, Fut, E>(func: F) -> OwnedEventHandler<E>
where
    F: FnOnce(SuspendPolicy, Event) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<(), E>> + Send + 'static,
{
    OwnedEventHandler::new(func)
}

#[cfg(test)]
mod tests {
    use crate::events::*;
    use std::convert::Infallible;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use jdwp_types::SuspendPolicy;

    #[tokio::test]
    async fn test_async_handle_event() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let rx = Arc::new(Mutex::new(rx));
        let mut func = |policy: SuspendPolicy, event: Event| async move {
            let mut guard = rx.lock().await;
            let _ = guard.recv().await;
            Result::<_, Infallible>::Ok(())
        };
        tx.send(()).unwrap();
        let fut = func.handle_event(SuspendPolicy::All, Event::VmDisconnected);
        fut.await.expect("Failed to receive event");
    }

    #[tokio::test]
    async fn test_async_owned_handle_event() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let rx = Arc::new(Mutex::new(rx));
        let mut func = handle_event(|suspend_policy: SuspendPolicy, event: Event| async move {
            let mut guard = rx.lock().await;
            let _ = guard.recv().await;
            Result::<_, Infallible>::Ok(())
        });
        tx.send(()).unwrap();

        let fut = func.handle_event(SuspendPolicy::All, Event::VmDisconnected);
        fut.await.expect("Failed to receive event");
    }
}
