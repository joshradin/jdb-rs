//! # `jdwp-types`
//! Provides all the JDWP types as defined by the [jdwp-spec].
//!
//! [jdwp-spec]: https://docs.oracle.com/javase/8/docs/technotes/guides/jpda/jdwp-spec.html

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::ffi::c_double;
pub use ids::*;
pub use constants::*;
use thiserror::Error;
use crate::private::Repr;

mod ids;
mod macros;
mod constants;

/// A byte value
pub type Byte = u8;
impl JdwpValue for Byte {}
/// A boolean value, encoded as 0 for false and non-zero for true
pub type Boolean = u8;
/// A four-byte integer value
pub type Int = i32;
impl JdwpValue for Int {}
/// A eight-byte integer value
pub type Long = i64;
impl JdwpValue for Long {}

/// An executable location. The location is identified by one byte type tag followed by a classID
/// followed by a methodID followed by an unsigned eight-byte index, which identifies the location
/// within the method. Index values are restricted as follows:
///
///  - The index of the start location for the method is less than all other locations in the method.
///  - The index of the end location for the method is greater than all other locations in the method.
///  - If a line number table exists for a method, locations that belong to a particular line must
///     fall between the line's location index and the location index of the next line in the table.
///
/// Index values within a method are monotonically increasing from the first executable point in the
/// method to the last. For many implementations, each byte-code instruction in the method has its
/// own index, but this is not required.
///
/// The type tag is necessary to identify whether location's classID identifies a class or an
/// interface. Almost all locations are within classes, but it is possible to have executable code
/// in the static initializer of an interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Location {
    /// Type tag
    pub tag: TypeTag,
    /// The owning class
    pub class: ClassId,
    /// The owning method
    pub method: MethodId,
    /// Offset within the method
    pub offset: u64,
}

/// Represents any Jdwp value, either a primitive or an Id type
pub trait JdwpValue {}

/// Any value
#[derive(Debug, Clone)]
pub enum Value {
    Array(ArrayId),
    Byte(Byte),
    Boolean(bool),
    Char(u16),
    Object(ObjectId),
    Float(f32),
    Double(f64),
    Int(i32),
    Long(i64),
    Short(i16),
    Void,
    String(StringId),
    Thread(ThreadId),
    ThreadGroup(ThreadGroupId),
    ClassLoader(ClassLoaderId),
    ClassObject(ClassObjectId)
}

/// Unknown tag constant
#[derive(Debug, Error)]
#[error("Unknown tag constant: {0}")]
pub struct UnknownTagError<T : Repr>(T);

mod private {
    use std::fmt::Display;

    pub trait Identifiable {}
    pub trait Repr : Display {}

    impl Repr for u8{}
    impl Repr for u16 {}
}

#[cfg(test)]
mod tests {
    use crate::ids::{ArrayId, TaggedObjectId, ThreadId};

    #[test]
    fn test_tagged_object_conversion() {
        let original = ArrayId::new(101);
        let tagged = TaggedObjectId::from(original);
        let _converted: ArrayId = tagged
            .try_into()
            .expect("converting to ArrayId should not fail");
        ThreadId::try_from(tagged)
            .expect_err("converting to ThreadId should fail because its an object id");
    }
}

