use crate::codec::{JdwpDecodable, JdwpEncodable};
use crate::raw::packet::CommandData;

/// used for representing a JDWP command
pub trait JdwpCommand: Sized + JdwpEncodable {
    type Reply: JdwpDecodable;

    fn command_data() -> CommandData;
}
