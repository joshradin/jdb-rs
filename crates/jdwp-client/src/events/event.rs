use thiserror::Error;
use crate::id_sizes::IdSizes;
use crate::raw::packet::RawCommandPacket;

/// Events codec
#[derive(Debug, Clone, Copy)]
pub struct EventsCodec {
    id_sizes: IdSizes,
}

/// Events, as received by the JVM
#[derive(Debug, Clone, Copy)]
pub enum Event {
    VmDeath,
}

impl TryFrom<RawCommandPacket> for Event {
    type Error = NotAnEventError;

    fn try_from(value: RawCommandPacket) -> Result<Self, Self::Error> {
        if !(value.header().command().command_set() == 64 && value.header().command().command() == 100) {
            return Err(NotAnEventError);
        }

        todo!("event translation")
    }
}

#[derive(Debug, Error)]
#[error("The given raw command packet is not an event")]
pub struct NotAnEventError;