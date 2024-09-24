//! All JDB commands

use crate::codec::{DecodeJdwpDataError, JdwpDecodable, JdwpDecoder, JdwpEncodable};
use crate::packet::JdwpCommand;
use crate::raw::packet::CommandData;
use jdwp_types::Int;

/// Gets the version of the JVM connected to
#[derive(Debug)]
pub struct Version;

impl JdwpCommand for Version {
    type Reply = VersionReply;

    fn command_data() -> CommandData {
        CommandData::new(1, 1)
    }
}
impl JdwpEncodable for Version {}

#[derive(Debug)]
pub struct VersionReply {
    pub description: String,
    pub major: Int,
    pub minor: Int,
    pub version: String,
    pub name: String,
}

impl JdwpDecodable for VersionReply {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        Ok(Self {
            description: decoder.get()?,
            major: decoder.get()?,
            minor: decoder.get()?,
            version: decoder.get()?,
            name: decoder.get()?,
        })
    }
}
