use bitfield::bitfield;
use bytes::Bytes;
use private::Sealed;

pub const MAX_PACKET_LENGTH: usize = 1 << 22;
pub const MIN_PACKET_LENGTH: usize = size_of::<u32>() * 2 + size_of::<u8>() + size_of::<u16>();

bitfield! {
    #[derive(Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct Flags(u8);
    impl Debug;

    pub is_reply, set_is_reply: 7;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CommandData {
    command_set: u8,
    command: u8,
}

impl CommandData {
    /// Creates a new command data struct
    pub fn new(command_set: u8, command: u8) -> CommandData {
        Self {
            command_set,
            command,
        }
    }

    /// Gets the command set
    pub fn command_set(&self) -> u8 {
        self.command_set
    }

    /// Gets the command
    pub fn command(&self) -> u8 {
        self.command
    }
}
impl Sealed for CommandData {}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ErrorCode {
    code: u16,
}

impl ErrorCode {
    pub fn new(code: u16) -> ErrorCode {
        Self { code }
    }

    /// Gets the code
    pub fn code(&self) -> u16 {
        self.code
    }
}

impl Sealed for ErrorCode {}


/// All valid header variable data must be representable by a u16
pub trait HeaderVariableData: Sealed {
    fn from_u16(value: u16) -> Self;
    fn to_u16(&self) -> u16;
}

impl Flags {
    const fn new_command() -> Self {
        Flags(0)
    }

    const fn new_reply() -> Self {
        Flags(0x80)
    }
}

/// A command packet
pub type RawCommandPacket = RawPacket<CommandData>;
/// A reply packet
pub type RawReplyPacket = RawPacket<ErrorCode>;

/// Any packet
#[derive(Debug, Clone)]
pub enum AnyRawPacket {
    /// Command packet
    Command(RawCommandPacket),
    /// Reply packet
    Reply(RawReplyPacket),
}

/// An arbitrary packet type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPacket<T: HeaderVariableData> {
    header: Header<T>,
    data: Bytes
}

impl<T: HeaderVariableData> RawPacket<T> {
    /// Gets the header for this packet
    pub fn header(&self) -> &Header<T> {
        &self.header
    }
    /// Gets the data for this packet
    pub fn data(&self) -> &Bytes {
        &self.data
    }
}

impl RawCommandPacket {
    pub fn new_command(id: u32, command: CommandData, data: Bytes) -> Self {
        let length = (MIN_PACKET_LENGTH + data.len()) as u32;
        Self {
            header: Header {
                length,
                id,
                flags: Flags::new_command(),
                var: command,
            },
            data
        }
    }
}

impl RawReplyPacket {
    pub fn new_reply(id: u32, error_code: ErrorCode, data: Bytes) -> Self {
        let length = (MIN_PACKET_LENGTH + data.len()) as u32;
        Self {
            header: Header {
                length,
                id,
                flags: Flags::new_reply(),
                var: error_code,
            },
            data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Header<T: HeaderVariableData> {
    length: u32,
    id: u32,
    flags: Flags,
    var: T,
}

impl<T: HeaderVariableData> Header<T> {
    /// Gets the total length of the packet
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Gets the id of the packet
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Gets the flags (if set) for this packet
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Gets the variable data as a generic type
    pub fn var(&self) -> &T {
        &self.var
    }
}

impl Header<CommandData> {
    /// Gets the command for this header
    pub fn command(&self) -> CommandData {
        self.var
    }
}

impl Header<ErrorCode> {
    pub fn error_code(&self) -> ErrorCode {
        self.var
    }
}

impl HeaderVariableData for CommandData {
    fn from_u16(value: u16) -> Self {
        let split: [u8; 2] = value.to_be_bytes();
        Self {
            command_set: split[0],
            command: split[1],
        }
    }

    fn to_u16(&self) -> u16 {
        let joined: [u8; 2] = [self.command_set,self.command];
        u16::from_be_bytes(joined)
    }
}

impl HeaderVariableData for ErrorCode {
    fn from_u16(value: u16) -> Self {
        Self { code: value }
    }

    fn to_u16(&self) -> u16 {
        self.code
    }
}

pub mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use crate::raw::packet::Flags;

    #[test]
    fn test_flag_invariants() {
        let command = Flags::new_command();
        assert!(!command.is_reply());
        let reply = Flags::new_reply();
        assert!(reply.is_reply());
    }
}
