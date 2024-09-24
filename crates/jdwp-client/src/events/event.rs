use crate::codec::{DecodeJdwpDataError, JdwpCodec, JdwpDecodable, JdwpDecoder};
use crate::raw::packet::RawCommandPacket;
use jdwp_types::{
    Boolean, Byte, ClassStatus, EventKind, FieldId, Int, JdwpValue, Location, Long, ObjectId,
    ReferenceTypeId, SuspendPolicy, TaggedObjectId, ThreadId, TypeTag, Value,
};
use std::io;
use std::io::ErrorKind;
use thiserror::Error;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct Events {
    pub policy: SuspendPolicy,
    pub events: Vec<Event>,
}

/// Events, as received by the JVM
#[derive(Debug, Clone)]
pub enum Event {
    SingleStep {
        request_id: Int,
        thread: ThreadId,
        location: Location,
    },
    Breakpoint {
        request_id: Int,
        thread: ThreadId,
        location: Location,
    },
    FramePop,
    Exception {
        request_id: Int,
        thread: ThreadId,
        location: Location,
        exception: TaggedObjectId,
        catch_location: Location,
    },
    UserDefined,
    ThreadStart {
        request_id: Int,
        thread: ThreadId,
    },
    ThreadDeath {
        request_id: Int,
        thread: ThreadId,
    },
    ClassPrepare {
        request_id: Int,
        thread: ThreadId,
        ref_type_tag: TypeTag,
        type_id: ReferenceTypeId,
        signature: String,
        status: ClassStatus,
    },
    ClassUnload {
        request_id: Int,
        signature: String,
    },
    ClassLoad,
    FieldAccess {
        request_id: Int,
        thread: ThreadId,
        ref_type_tag: TypeTag,
        type_id: ReferenceTypeId,
        field_id: FieldId,
        object: ObjectId,
    },
    FieldModification {
        request_id: Int,
        thread: ThreadId,
        ref_type_tag: TypeTag,
        type_id: ReferenceTypeId,
        field_id: FieldId,
        object: ObjectId,
        value_to_be: Value,
    },
    ExceptionCatch,
    MethodEntry {
        request_id: Int,
        thread: ThreadId,
        location: Location,
    },
    MethodExit {
        request_id: Int,
        thread: ThreadId,
        location: Location,
    },
    MethodExitWithReturnValue {
        request_id: Int,
        thread: ThreadId,
        location: Location,
        value: Value,
    },
    MonitorContendedEnter {
        request_id: Int,
        thread: ThreadId,
        object: TaggedObjectId,
        location: Location,
    },
    MonitorContendedEntered {
        request_id: Int,
        thread: ThreadId,
        object: TaggedObjectId,
        location: Location,
    },
    MonitorWait {
        request_id: Int,
        thread: ThreadId,
        object: TaggedObjectId,
        location: Location,
        timeout: Long,
    },
    MonitorWaited {
        request_id: Int,
        thread: ThreadId,
        object: TaggedObjectId,
        location: Location,
        timed_out: Boolean,
    },
    VmStart {
        request_id: Int,
        thread: ThreadId,
    },
    VmDeath {
        request_id: Int,
    },
    /// Never sent across JDWP
    VmDisconnected,
}

pub(crate) fn to_events(
    command: RawCommandPacket,
    events_codec: &JdwpCodec,
) -> Result<Events, io::Error> {
    if !(command.header().command().command_set() == 64
        && command.header().command().command() == 100)
    {
        return Err(io::Error::new(ErrorKind::InvalidData, NotAnEventError));
    }
    let mut decoder = JdwpDecoder::new(events_codec, command.data().clone());
    let policy_raw = decoder
        .get::<Byte>()
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, NotAnEventError))?;
    let policy = SuspendPolicy::try_from(policy_raw)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, NotAnEventError))?;

    trace!("got events with policy: {policy:?}");

    let events = decoder
        .get::<Vec<Event>>()
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, NotAnEventError))?;

    Ok(Events { policy, events })
}

impl JdwpDecodable for Event {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        let event_kind = decoder
            .get::<Byte>()
            .and_then(|i| EventKind::try_from(i).map_err(|e| e.into()))?;
        trace!("got event kind: {event_kind:?}");
        let event: Event = match event_kind {
            EventKind::SingleStep => Event::SingleStep {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::Breakpoint => Event::Breakpoint {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::FramePop => Event::FramePop,
            EventKind::Exception => Event::Exception {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
                exception: decoder.get()?,
                catch_location: decoder.get()?,
            },
            EventKind::UserDefined => Event::UserDefined,
            EventKind::ThreadStart => Event::ThreadStart {
                request_id: decoder.get()?,
                thread: decoder.get()?,
            },
            EventKind::ThreadDeath => Event::ThreadDeath {
                request_id: decoder.get()?,
                thread: decoder.get()?,
            },
            EventKind::ClassPrepare => Event::ClassPrepare {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                ref_type_tag: decoder
                    .get::<Byte>()
                    .and_then(|b| Ok(TypeTag::try_from(b)?))?,
                type_id: decoder.get()?,
                signature: decoder.get()?,
                status: decoder.get::<Byte>()
                               .and_then(|b| Ok(ClassStatus::try_from(b)?))?,
            },
            EventKind::ClassUnload => Event::ClassUnload {
                request_id: decoder.get()?,
                signature: decoder.get()?,
            },
            EventKind::ClassLoad => Event::ClassLoad,
            EventKind::FieldAccess => Event::FieldAccess {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                ref_type_tag: decoder.get::<Byte>()
                                     .and_then(|b| Ok(TypeTag::try_from(b)?))?,
                type_id: decoder.get()?,
                field_id: decoder.get()?,
                object: decoder.get()?,
            },
            EventKind::FieldModification => Event::FieldModification {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                ref_type_tag: decoder.get::<Byte>()
                                     .and_then(|b| Ok(TypeTag::try_from(b)?))?,
                type_id: decoder.get()?,
                field_id: decoder.get()?,
                object: decoder.get()?,
                value_to_be: decoder.get()?,
            },
            EventKind::ExceptionCatch => Event::ExceptionCatch,
            EventKind::MethodEntry => Event::MethodEntry {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::MethodExit => Event::MethodExit {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::MethodExitWithReturnValue => Event::MethodExitWithReturnValue {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                location: decoder.get()?,
                value: decoder.get()?,
            },
            EventKind::MonitorContendedEnter => Event::MonitorContendedEnter {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                object: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::MonitorContendedEntered => Event::MonitorContendedEnter {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                object: decoder.get()?,
                location: decoder.get()?,
            },
            EventKind::MonitorWait => Event::MonitorWait {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                object: decoder.get()?,
                location: decoder.get()?,
                timeout: decoder.get()?,
            },
            EventKind::MonitorWaited => Event::MonitorWaited {
                request_id: decoder.get()?,
                thread: decoder.get()?,
                object: decoder.get()?,
                location: decoder.get()?,
                timed_out: decoder.get()?,
            },
            EventKind::VmStart => Event::VmStart {
                request_id: decoder.get()?,
                thread: decoder.get()?,
            },
            EventKind::VmDeath => Event::VmDeath {
                request_id: decoder.get()?,
            },
            EventKind::VmDisconnected => {
                unreachable!()
            }
        };
        Ok(event)
    }
}

#[derive(Debug, Error)]
#[error("The given raw command packet is not an event")]
pub struct NotAnEventError;
