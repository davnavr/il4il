//! Provides a model of the IL4IL instruction set.

pub mod value;

mod block;

pub use block::Block;
pub use value::Value;

/// Represents an IL4IL instruction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Instruction {
    /// Indicates that control flow cannot reach this particular location, causing undefined behavior otherwise.
    ///
    /// ## Assembly Syntax
    ///
    /// ```text
    /// unreachable
    /// ```
    Unreachable,
    /// Transfers control flow back to the calling function, providing the specified return value(s).
    ///
    /// ### Assembly Syntax
    /// ```text
    /// return <value0>, <value1>, ... ; Return multiple values
    /// return ; Return no values
    /// ```
    Return(Box<[Value]>),
}

impl Instruction {
    /// Returns `true` if this [`Instruction`] can only be used at the end of a [`Block`].
    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Unreachable | Self::Return(_))
    }
}

macro_rules! opcode {
    {$($name:ident = $code:literal,)*} => {
        /// Specifies an IL4IL instruction.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum Opcode {
            $($name,)*
        }

        impl From<Opcode> for crate::integer::VarU28 {
            fn from(opcode: Opcode) -> Self {
                Self::from(opcode as u8)
            }
        }

        impl Instruction {
            pub fn opcode(&self) -> Opcode {
                match self {
                    $(Self::$name { .. } => Opcode::$name,)*
                }
            }
        }
    };
}

opcode! {
    Unreachable = 0,
    Return = 1,
}
