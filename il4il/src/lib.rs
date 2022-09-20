//! Provides a reader, writer, and validator for IL4IL modules.

#![deny(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

pub mod binary;
pub mod disassemble;
pub mod function;
pub mod identifier;
pub mod index;
pub mod instruction;
pub mod integer;
pub mod module;
pub mod symbol;
pub mod type_system;
pub mod validation;
pub mod versioning;

pub use error_stack;

#[cfg(test)]
use il4il_propcheck as propcheck;

#[macro_export]
#[doc(hidden)]
macro_rules! kind_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident : $inty:ty {
        $($(#[$case_meta:meta])* $case_name:ident = $case_number:literal,)*
    }) => {
        $(#[$meta])*
        #[repr(u8)]
        $vis enum $name {
            $($(#[$case_meta])* $case_name = $case_number,)*
        }

        impl $name {
            pub const fn new(value: $inty) -> Option<Self> {
                match value {
                    $(_ if value == $case_number => Some(Self::$case_name),)*
                    _ => None
                }
            }
        }

        impl From<$name> for $inty {
            fn from(kind: $name) -> Self {
                match kind {
                    $($name::$case_name => $case_number,)*
                }
            }
        }
    };
}
