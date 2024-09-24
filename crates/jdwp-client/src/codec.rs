//! The codec for encoding and decoding packets, using [tokio-util]'s [Encoder] and [Decoder] traits

use std::error::Error;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use crate::id_sizes::IdSizes;
use crate::packet::JdwpCommand;
use bytes::{Buf, Bytes, BytesMut};
use jdwp_types::*;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use tracing::trace;

/// A codec for encoding and decoding JDWP packets
#[derive(Debug, Default)]
pub struct JdwpCodec {
    id_sizes: IdSizes,
}

impl JdwpCodec {
    pub fn id_sizes(&self) -> IdSizes {
        self.id_sizes
    }
}

/// Encodable into bytes
pub trait JdwpEncodable {
    /// Encodes this into a buffer
    fn encode(&self, codec: &JdwpCodec, buffer: &mut BytesMut) {}
}

/// Decodable from a byte buffer
pub trait JdwpDecodable: Sized {
    type Err : Error;

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

impl JdwpDecodable for Int {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        if decoder.data.len() < 4 {
            return Err(DecodeJdwpDataError::NotEnoughBytes);
        }
        Ok(decoder.data.get_i32())
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

macro_rules! decode_id {
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
            )*
        )*
    };
}

decode_id! {
    ObjectId, ThreadId, ThreadGroupId, StringId, ClassLoaderId, ClassObjectId,
        ArrayId, ReferenceTypeId, ClassId, InterfaceId, ArrayTypeId: object_id_size;
    MethodId: method_id_size;
    FieldId: field_id_size;
    FrameId: frame_id_size;
}

impl<T: JdwpDecodable<Err =DecodeJdwpDataError>> JdwpDecodable for Vec<T> {
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
    Utf8DecodeError(#[from] FromUtf8Error)
}
