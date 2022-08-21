//! Provides a model of the IL4IL instruction set.

mod block;

pub use block::Block;

/// Represents an IL4IL instruction.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
}

impl Instruction {
    /// Returns `true` if this [`Instruction`] can only be used at the end of a [`Block`].
    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Unreachable)
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
}
