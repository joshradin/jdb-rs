use jdwp_client_tests::JavaInstance;
use std::io;
use std::time::Duration;
use tracing::info;

#[test_log::test(tokio::test)]
async fn test_connect() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    client.on_event(|policy, e| async move {
        info!("event [policy: {policy:?}]: {e:?}");
        Ok(())
    }).await;
    client.dispose().await?;
    Ok(())
}


#[test_log::test(tokio::test)]
async fn test_disconnect() -> io::Result<()> {
    let java_instance = JavaInstance::new(0, "BusyBeaver").await?;
    println!("started java instance");
    let mut client = java_instance.connect().await?;
    client.on_event(|policy, e| async move {
        info!("event [policy: {policy:?}]: {e:?}");
        Ok(())
    }).await;
    drop(java_instance);
    client.dispose().await?;
    Ok(())
}
