/// used for easily creating tagged types
macro_rules! tagged_type {
    (
        repr: $repr_ty:ty;
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$id_attr:meta])*
                $var:ident = $val:expr
            ),+ $(,)?
        }
    ) => {
        #[repr($repr_ty)]
        $(#[$attr])*
        $vis enum $name {
            $(
                $(#[$id_attr])*
                $var = $val,
            )*
        }

        impl From<$name> for $repr_ty {
            fn from(var: $name) -> Self {
                var as $repr_ty
            }
        }

        impl TryFrom<$repr_ty> for $name {
            type Error = $crate::UnknownTagError<$repr_ty>;

            fn try_from(value: $repr_ty) -> Result<Self, Self::Error> {
                let tag = match value {
                    $(
                    $val => $name::$var,
                    )*
                    unknown => return Err($crate::UnknownTagError(unknown))
                };
                Ok(tag)
            }
        }
    };
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$id_attr:meta])*
                $var:ident = $val:expr
            ),+ $(,)?
        }
    ) => {
        $crate::macros::tagged_type! {
            repr: u8;
            $(#[$attr])*
            $vis enum $name {
                $(
                    $(#[$id_attr])*
                    $var = $val
                ),*
            }
        }
    };
}

pub(crate) use tagged_type;