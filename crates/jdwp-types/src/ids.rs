use std::any::type_name;
use thiserror::Error;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter, Pointer};
use std::marker::PhantomData;

/// Uniquely identifies an object in the target VM.
///
/// A particular object will be identified by exactly one ObjectId in JDWP commands and replies through
/// its lifetime (or until it's explicitly disposed). An ObjectId of 0 represents a null object
pub type ObjectId = Id<Object>;
/// Uniquely identifies an object in the target VM that is known to be a thread
pub type ThreadId = Id<Thread>;
/// Uniquely identifies an object in the target VM that is known to be a thread group
pub type ThreadGroupId = Id<ThreadGroup>;
/// Uniquely identifies an object in the target VM that is known to be a string object. Note: this is very different from string, which is a value.
pub type StringId = Id<String>;
/// Uniquely identifies an object in the target VM that is known to be a class loader object
pub type ClassLoaderId = Id<ClassLoader>;
/// Uniquely identifies an object in the target VM that is known to be a class object.
pub type ClassObjectId = Id<ClassObject>;
/// Uniquely identifies an object in the target VM that is known to be an array.
pub type ArrayId = Id<Array>;
/// Uniquely identifies a reference type in the target VM. It should not be assumed that for a particular class,
/// the classObjectID and the referenceTypeID are the same. A particular reference type will be identified by
/// exactly one ID in JDWP commands and replies throughout its lifetime A referenceTypeID is not reused to identify a different reference type,
/// regardless of whether the referenced class has been unloaded.
pub type ReferenceTypeId = Id<ReferenceType>;
/// Uniquely identifies a reference type in the target VM that is known to be a class type.
pub type ClassId = Id<Class>;
/// Uniquely identifies a reference type in the target VM that is known to be an interface type.
pub type InterfaceId = Id<Interface>;
/// Uniquely identifies a reference type in the target VM that is known to be an array type.
pub type ArrayTypeId = Id<ArrayType>;
/// Uniquely identifies a method in some class in the target VM. The methodID must uniquely identify
/// the method within its class/interface or any of its subclasses/subinterfaces/implementors. A
/// methodID is not necessarily unique on its own; it is always paired with a referenceTypeID to
/// uniquely identify one method. The referenceTypeID can identify either the declaring type of the
/// method or a subtype.
pub type MethodId = Id<Method>;
/// Uniquely identifies a field in some class in the target VM. The fieldID must uniquely identify
/// the field within its class/interface or any of its subclasses/subinterfaces/implementors. A
/// fieldID is not necessarily unique on its own; it is always paired with a referenceTypeID to
/// uniquely identify one field. The referenceTypeID can identify either the declaring type of the
/// field or a subtype.
pub type FieldId = Id<Field>;
/// Uniquely identifies a frame in the target VM. The frameID must uniquely identify the frame
/// within the entire VM (not only within a given thread). The frameID need only be valid during the
/// time its thread is suspended.
pub type FrameId = Id<Frame>;

impl<T: Identifiable> From<Id<T>> for u64 {
    fn from(value: Id<T>) -> Self {
        value.0
    }
}


/// Wrong type tag was found
#[derive(Debug, Error)]
#[error("Tag mismatch, expected {expected:?} but got {got:?}")]
pub struct TagTypeMismatchError {
    expected: Tag,
    got: Tag,
}

/// An error occurred while trying to convert a tagged object
#[derive(Debug, Error)]
pub enum TaggedObjectConversionError {
    /// Unknown tag error
    #[error(transparent)]
    UnknownTag(#[from] UnknownTagError<u8>),
    /// Tag type mismatch error
    #[error(transparent)]
    TagTypeMismatch(#[from] TagTypeMismatchError),
}

/// A tagged object Id
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TaggedObjectId(Tag, Id<Unknown>);

impl JdwpValue for TaggedObjectId {}

impl TaggedObjectId {
    /// Gets the tag for this object id
    pub fn tag(&self) -> Tag {
        self.0
    }

    /// Gets the unknown id type
    pub fn id(&self) -> Id<Unknown> {
        self.1
    }
}

impl PartialOrd for TaggedObjectId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.eq(&other.0) {
            Some(self.1.cmp(&other.1))
        } else {
            None
        }
    }
}

/// Uniquely identifies some *thing* in the target VM
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id<T: Identifiable>(u64, PhantomData<T>);



impl<T: Identifiable> Id<T> {
    /// Creates a new [Id]. Since the maximum byte size of an [Id] is an u64, we can always use that as
    /// input for creating one. However, there's no guarantee that this [Id] is valid within the given
    /// VM's context.
    pub const fn new(id: u64) -> Self {
        Id(id, PhantomData)
    }

    /// Gets the ids
    pub const fn get(&self) -> u64 {
        self.0
    }
}

impl<T: Identifiable> JdwpValue for Id<T> {
}

impl<T: Identifiable + Debug> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(format!("Id<{}>", type_name::<T>()).as_str())
            .field(&self.0)
            .finish()
    }
}


macro_rules! type_to_tagged {
    ($(
        $ty:ty: $tag:expr
    )*) => {
        $(
            impl From<$ty> for TaggedObjectId {
                fn from(value: $ty) -> Self {
                    TaggedObjectId($tag, Id(value.0, PhantomData))
                }
            }

            impl TryFrom<TaggedObjectId> for $ty {
                type Error = TagTypeMismatchError;

                fn try_from(value: TaggedObjectId) -> Result<Self, Self::Error> {
                    if value.0 == $tag {
                        let inner = value.1.0;
                        Ok(
                            Id(inner, PhantomData)
                        )
                    } else {
                        Err(
                            TagTypeMismatchError { expected: $tag, got: value.0 }
                        )
                    }
                }
            }
        )*
    };
}

type_to_tagged!(
    ObjectId: Tag::Object
    ThreadId: Tag::Thread
    ThreadGroupId: Tag::ThreadGroup
    StringId: Tag::String
    ClassLoaderId: Tag::ClassLoader
    ClassObjectId: Tag::ClassObject
    ArrayId: Tag::Array
);



use crate::private::Identifiable;
pub use identifiable_types::*;
use crate::{JdwpValue, UnknownTagError};
use crate::constants::Tag;

mod identifiable_types {
    use crate::private::Identifiable;

    macro_rules! identifiables {
        ($($ty:ident)*) => {
            $(
                /// An identifiable type
                #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
                pub enum $ty {}
                impl Identifiable for $ty {}
            )*
        };
    }

    identifiables!(
         Unknown Object Thread ThreadGroup String ClassLoader ClassObject Array ReferenceType Class Interface ArrayType
         Method Field Frame
    );
}
