use std::io::ErrorKind;
use tokio_util::codec::{Decoder, Encoder};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tracing::{instrument, trace};
use crate::raw::packet::{AnyRawPacket, CommandData, ErrorCode, Flags, HeaderVariableData, RawCommandPacket, RawPacket, RawReplyPacket, MAX_PACKET_LENGTH, MIN_PACKET_LENGTH};

/// Codec for encoding and decoding jdwp packets
#[derive(Debug, Default, Copy, Clone)]
pub struct RawCodec;

impl<T: HeaderVariableData> Encoder<RawPacket<T>> for RawCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: RawPacket<T>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut limited_buffer = dst.limit(item.header().length() as usize);
        limited_buffer.put_u32(item.header().length());
        limited_buffer.put_u32(item.header().id());
        limited_buffer.put_u8(item.header().flags().0);
        limited_buffer.put_u16(item.header().var().to_u16());
        limited_buffer.put(item.data());
        Ok(())
    }
}

impl Decoder for RawCodec {
    type Item = AnyRawPacket;
    type Error = std::io::Error;

    #[instrument(skip_all, fields(buffered=src.len()))]
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            trace!("current length of {} is not enough to read length of packet", src.len());
            return Ok(None);
        }
        let length = u32::from_be_bytes(src[..4].try_into().unwrap()) as usize;
        trace!("got length for packet: {length}");
        if length > MAX_PACKET_LENGTH {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("{} is larger than max packet size: {}", length, MAX_PACKET_LENGTH),
            ))
        }
        if src.len() < length {
            trace!("current length of {} is not enough to read length of packet", src.len());
            src.reserve(length - src.len());
            return Ok(None);
        }
        src.advance(4);

        let id = src.get_u32();
        trace!("got packet id: {id}");
        let raw_flag = src.get_u8();
        let flag = Flags(raw_flag);
        trace!("got flag: {flag:?}");
        let raw_var = src.get_u16();
        let data = src[..length - MIN_PACKET_LENGTH].to_vec();
        let packet = if flag.is_reply() {
            let error_code = ErrorCode::from_u16(raw_var);
            AnyRawPacket::Reply(RawReplyPacket::new_reply(id, error_code, data))
        } else {
            let command = CommandData::from_u16(raw_var);
            AnyRawPacket::Command(RawCommandPacket::new_command(id, command, data))
        };
        Ok(Some(packet))
    }
}