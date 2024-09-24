//! All JDB commands

use crate::codec::{
    DecodeJdwpDataError, JdwpDecodable, JdwpDecoder, JdwpEncodable, JdwpEncoder,
};
use crate::packet::JdwpCommand;
use crate::raw::packet::CommandData;
use bytes::BufMut;
use jdwp_types::{Byte, ClassStatus, Int, ReferenceTypeId, ThreadGroupId, ThreadId, TypeTag};
use tracing::instrument;

macro_rules! command {
    (
        command_set: $command_set:expr;
        command: $command:expr;
        $(#[$meta:meta])*
        $vis:vis struct $command_id:ident {
            $($field_vis:vis $field:ident: $field_ty:ty),*
            $(,)?
        } -> {
            $(
                $reply_field_vis:vis $reply_field:ident: $reply_field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        paste::paste! {
            $(#[$meta])*
            $vis struct $command_id
                {
                    $(
                        $field_vis $field: $field_ty
                    )*
                }


            impl JdwpEncodable for $command_id {
                fn encode(&self, encoder: &mut JdwpEncoder) {
                    $(
                        encoder.put(&self.$field);
                    )*
                }
            }

            impl JdwpCommand for $command_id {
                type Reply = [<$command_id Reply>];

                fn command_data() -> CommandData {
                    CommandData::new($command_set, $command)
                }
            }

            $(#[$meta])*
            $vis struct [<$command_id Reply>] {
            $(
                $reply_field_vis $reply_field: $reply_field_ty,
            )*
            }

            impl JdwpDecodable for [<$command_id Reply>] {
                type Err = DecodeJdwpDataError;

                fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
                    Ok(Self {
                        $(
                            $reply_field: decoder.get()?
                        ),*

                    })
                }
            }
        }
    };
    (
        command_set: $command_set:expr;
        command: $command:expr;
        $(#[$meta:meta])*
        $vis:vis struct $command_id:ident -> {
            $(
                $reply_field_vis:vis $reply_field:ident: $reply_field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        paste::paste! {
            $(#[$meta])*
            $vis struct $command_id;

            impl JdwpEncodable for $command_id {

            }

            impl JdwpCommand for $command_id {
                type Reply = [<$command_id Reply>];

                fn command_data() -> CommandData {
                    CommandData::new($command_set, $command)
                }
            }

            $(#[$meta])*
            $vis struct [<$command_id Reply>] {
            $(
                $reply_field_vis $reply_field: $reply_field_ty,
            )*
            }

            impl JdwpDecodable for [<$command_id Reply>] {
                type Err = DecodeJdwpDataError;

                fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
                    Ok(Self {
                        $(
                            $reply_field: decoder.get()?
                        ),*

                    })
                }
            }
        }
    };
    (
        command_set: $command_set:expr;
        command: $command:expr;
        $(#[$meta:meta])*
        $vis:vis struct $command_id:ident;
    ) => {
        paste::paste! {
            $(#[$meta])*
            $vis struct $command_id;

            impl JdwpEncodable for $command_id {

            }

            impl JdwpCommand for $command_id {
                type Reply = [<$command_id Reply>];

                fn command_data() -> CommandData {
                    CommandData::new($command_set, $command)
                }
            }

            $(#[$meta])*
            $vis struct [<$command_id Reply>];

            impl JdwpDecodable for [<$command_id Reply>] {
                type Err = DecodeJdwpDataError;

                fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
                    Ok(Self)
                }
            }
        }
    };
}

command! {
    command_set: 1;
    command: 1;
    /// Gets the version of the JVM connected to
    #[derive(Debug)]
    pub struct Version -> {
        pub description: String,
        pub major: Int,
        pub minor: Int,
        pub version: String,
        pub name: String,
    }
}

command! {
    command_set: 1;
    command: 2;
    /// Gets all classes by a given jni signature
    #[derive(Debug)]
    pub struct ClassesBySignatures {
        pub signature: String
    } -> {
        pub classes: Vec<ClassReference>
    }
}

#[derive(Debug)]
pub struct ClassReference {
    pub type_tag: TypeTag,
    pub id: ReferenceTypeId,
    pub status: ClassStatus,
}

impl JdwpDecodable for ClassReference {
    type Err = DecodeJdwpDataError;

    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        Ok(Self {
            type_tag: decoder
                .get::<Byte>()
                .and_then(|i| Ok(TypeTag::try_from(i)?))?,
            id: decoder.get()?,
            status: decoder.get::<ClassStatus>()?,
        })
    }
}

command! {
    command_set: 1;
    command: 3;
    /// Gets all classes by a given jni signature
    #[derive(Debug)]
    pub struct AllClasses -> {
        pub classes: Vec<ClassReferenceWithSignature>
    }
}

#[derive(Debug)]
pub struct ClassReferenceWithSignature {
    pub type_tag: TypeTag,
    pub id: ReferenceTypeId,
    pub signature: String,
    pub status: ClassStatus,
}

impl JdwpDecodable for ClassReferenceWithSignature {
    type Err = DecodeJdwpDataError;

    #[instrument(ret, skip_all)]
    fn decode(decoder: &mut JdwpDecoder) -> Result<Self, Self::Err> {
        Ok(Self {
            type_tag: decoder
                .get::<Byte>()
                .and_then(|type_tag_byte| Ok(TypeTag::try_from(type_tag_byte)?))?,
            id: decoder.get()?,
            signature: decoder.get()?,
            status: decoder.get::<ClassStatus>()?,
        })
    }
}

command! {
    command_set: 1;
    command: 4;
    #[derive(Debug)]
    pub struct AllThreads -> {
        pub threads: Vec<ThreadId>
    }
}

command! {
    command_set: 1;
    command: 5;
    #[derive(Debug)]
    pub struct TopLevelThreadGroups -> {
        pub groups: Vec<ThreadGroupId>
    }
}

command! {
    command_set: 1;
    command: 6;
    #[derive(Debug)]
    pub struct Dispose;
}


command! {
    command_set: 1;
    command: 7;
    pub struct IdSizes -> {
        pub field_id_size: Int,
        pub method_id_size: Int,
        pub object_id_size: Int,
        refernce_type_id_size: Int,
        pub frame_id_size: Int
    }
}
