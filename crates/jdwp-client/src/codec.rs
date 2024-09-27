//! The codec for encoding and decoding packets, using [tokio-util]'s [Encoder] and [Decoder] traits

use crate::id_sizes::IdSizes;
use crate::packet::JdwpCommand;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use jdwp_types::*;
use std::error::Error;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use tracing::trace;

/// A codec for encoding and decoding JDWP packets
#[derive(Debug, Default)]
pub struct JdwpCodec {
    id_sizes: IdSizes,
}

impl JdwpCodec {
    pub fn new(id_sizes: IdSizes) -> Self {
        Self { id_sizes }
    }

    pub fn id_sizes(&self) -> IdSizes {
        self.id_sizes
    }

    pub(crate) fn id_sizes_mut(&mut self) -> &mut IdSizes {
        &mut self.id_sizes
    }
}

/// Encodable into bytes
pub trait JdwpEncodable {
    /// Encodes this into a buffer
    fn encode(&self, encoder: &mut JdwpEncoder) {}
}

/// Decodable from a byte buffer
pub trait JdwpDecodable: Sized {
    type Err: Error;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err>;
}

impl JdwpDecodable for Byte {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        if decoder.data.len() < 1 {
            return Err(DecodeJdwpDataError::NotEnoughBytes);
        }
        Ok(decoder.data.get_u8())
    }
}
impl JdwpEncodable for Byte {
    fn encode(&self, encoder: &mut JdwpEncoder) {
        encoder.data.put_u8(*self);
    }
}

impl JdwpDecodable for Int {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        if decoder.data.len() < 4 {
            return Err(DecodeJdwpDataError::NotEnoughBytes);
        }
        Ok(decoder.data.get_i32())
    }
}

impl JdwpDecodable for ClassStatus {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        if decoder.data.len() < 4 {
            return Err(DecodeJdwpDataError::NotEnoughBytes);
        }
        let data = decoder.data.get_u32();
        Ok(ClassStatus(data))
    }
}

impl JdwpEncodable for Int {
    fn encode(&self, encoder: &mut JdwpEncoder) {
        encoder.data.put_i32(*self);
    }
}
impl JdwpDecodable for Long {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        if decoder.data.len() < 8 {
            return Err(DecodeJdwpDataError::NotEnoughBytes);
        }
        Ok(decoder.data.get_i64())
    }
}
impl JdwpEncodable for Long {
    fn encode(&self, encoder: &mut JdwpEncoder) {
        encoder.data.put_i64(*self);
    }
}
impl JdwpDecodable for TaggedObjectId {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl JdwpDecodable for std::string::String {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        let len = decoder.get::<Int>()? as usize;
        let bytes: Vec<u8> = Vec::from(&decoder.data[..len]);
        decoder.data.advance(len);
        let string = std::string::String::from_utf8(bytes)?;
        Ok(string)
    }
}

impl JdwpEncodable for std::string::String {
    fn encode(&self, encoder: &mut JdwpEncoder) {
        let len = self.len();
        encoder.data.put_i32(len as i32);
        encoder.data.put_slice(self.as_ref());
    }
}

impl JdwpDecodable for Location {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        let type_tag = decoder.get::<Byte>().and_then(|e| Ok(e.try_into()?))?;
        Ok(Location {
            tag: type_tag,
            class: decoder.get()?,
            method: decoder.get()?,
            offset: decoder.get::<Long>().map(|i| i as u64)?,
        })
    }
}

impl JdwpDecodable for Value {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        let tag = decoder.get::<Byte>().and_then(|b| Ok(Tag::try_from(b)?))?;

        let value = match tag {
            Tag::Array => Value::Array(decoder.get()?),
            Tag::Byte => Value::Byte(decoder.get()?),
            Tag::Char => Value::Char(decoder.data.get_u16()),
            Tag::Object => Value::Object(decoder.get()?),
            Tag::Float => Value::Float(decoder.data.get_f32()),
            Tag::Double => Value::Double(decoder.data.get_f64()),
            Tag::Int => Value::Int(decoder.get()?),
            Tag::Long => Value::Long(decoder.get()?),
            Tag::Short => Value::Short(decoder.data.get_i16()),
            Tag::Void => Value::Void,
            Tag::Boolean => Value::Boolean(decoder.data.get_u8() != 0),
            Tag::String => Value::String(decoder.get()?),
            Tag::Thread => Value::Thread(decoder.get()?),
            Tag::ThreadGroup => Value::ThreadGroup(decoder.get()?),
            Tag::ClassLoader => Value::ClassLoader(decoder.get()?),
            Tag::ClassObject => Value::ClassObject(decoder.get()?),
        };
        Ok(value)
    }
}

macro_rules! encdec_id {
    (
        $(
            $($id_type:ty),*: $id_size:ident
        );*
        $(;)?
    ) => {
        $(
            $(
                impl JdwpDecodable for $id_type {
                    type Err = DecodeJdwpDataError;

                    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
                        let len: usize = decoder.codec.id_sizes.$id_size();
                        let mut buffer = [0u8; 8];
                        if decoder.data.len() < len {
                            return Err(DecodeJdwpDataError::NotEnoughBytes);
                        }
                        decoder.data.copy_to_slice(&mut buffer[(8 - len)..]);
                        let decoded = u64::from_be_bytes(buffer);
                        Ok(Id::new(decoded))
                    }
                }

                impl JdwpEncodable for $id_type {
                    fn encode(&self, encoder: &mut JdwpEncoder) {
                        let len: usize = encoder.codec.id_sizes.$id_size();
                        let to_bytes = self.get().to_be_bytes();
                        let slice = &to_bytes[(8 - len)..][..len];
                        encoder.data.extend_from_slice(slice);
                    }

                }
            )*
        )*
    };
}

encdec_id! {
    ObjectId, ThreadId, ThreadGroupId, StringId, ClassLoaderId, ClassObjectId,
        ArrayId, ReferenceTypeId, ClassId, InterfaceId, ArrayTypeId: object_id_size;
    MethodId: method_id_size;
    FieldId: field_id_size;
    FrameId: frame_id_size;
}

impl<T: JdwpDecodable<Err = DecodeJdwpDataError>> JdwpDecodable for Vec<T> {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        let len = decoder.get::<Int>()?;
        if len < 0 {
            return Err(DecodeJdwpDataError::UnexpectedNegativeInt(len));
        }
        let mut collect = Vec::with_capacity(len as usize);
        trace!("getting {len} items");
        for _ in 0..len {
            let item = decoder.get::<T>()?;
            collect.push(item);
        }
        Ok(collect)
    }
}

#[derive(Debug)]
pub struct JdwpDecoder<'a> {
    pub(crate) codec: &'a JdwpCodec,
    pub(crate) data: Bytes,
}

impl<'a> JdwpDecoder<'a> {
    /// Creates a new decoder with a given codec
    pub fn new(codec: &'a JdwpCodec, data: Bytes) -> Self {
        Self {
            codec,
            data: Bytes::from(data),
        }
    }

    /// Decodes the next jdwp value
    pub fn get<T: JdwpDecodable>(&mut self) -> Result<T, T::Err> {
        T::decode(self)
    }
}

#[derive(Debug, Error)]
pub enum DecodeJdwpDataError {
    #[error("Not enough bytes to decode type")]
    NotEnoughBytes,
    #[error("Got negative integer {0} when only positive integers are expected")]
    UnexpectedNegativeInt(i32),
    #[error(transparent)]
    IllegalByteTag(#[from] UnknownTagError<u8>),
    #[error(transparent)]
    Utf8DecodeError(#[from] FromUtf8Error),
}

#[derive(Debug)]
pub struct JdwpEncoder<'a> {
    pub(crate) codec: &'a JdwpCodec,
    pub(crate) data: BytesMut,
}

impl<'a> JdwpEncoder<'a> {
    /// Creates a new decoder with a given codec
    pub fn new(codec: &'a JdwpCodec) -> Self {
        Self {
            codec,
            data: BytesMut::new(),
        }
    }

    /// Decodes the next jdwp value
    pub fn put<T: JdwpEncodable>(&mut self, to_encoded: &T) {
        to_encoded.encode(self)
    }
}

#[cfg(test)]
mod test {
    use crate::codec::{JdwpCodec, JdwpDecoder, JdwpEncoder};
    use crate::id_sizes::IdSizes;
    use jdwp_types::{Id, Object, ObjectId};

    #[test]
    fn encode_special_ids() {
        let id: Id<Object> = Id::new(101);
        let codec = JdwpCodec::new(IdSizes::new(6, 6, 6, 6));
        let mut encoder = JdwpEncoder::new(&codec);
        encoder.put(&id);
        assert_eq!(encoder.data.len(), 6);
        assert_eq!(&encoder.data[..], &[0, 0, 0, 0, 0, 101]);
        let mut decoder = JdwpDecoder::new(&codec, encoder.data.freeze());
        let decoded_id = decoder
            .get::<ObjectId>()
            .expect("could not decode objectid");
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn encode_special_ids_out_of_bounds() {
        let id: Id<Object> = Id::new(u64::MAX);
        let codec = JdwpCodec::new(IdSizes::new(6, 6, 6, 6));
        let mut encoder = JdwpEncoder::new(&codec);
        encoder.put(&id);
        assert_eq!(encoder.data.len(), 6);
        let mut decoder = JdwpDecoder::new(&codec, encoder.data.freeze());
        let decoded_id = decoder
            .get::<ObjectId>()
            .expect("could not decode objectid");
        assert_ne!(decoded_id, id);
        assert_eq!(decoded_id, Id::new(!(0xFFFF << 48)));
    }
}
