//! Provides a model of the IL4IL instruction set.

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

macro_rules! opcode {
    {
        $($name:ident = $code:literal,)*
    } => {
        /// Specifies an IL4IL instruction.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum Opcode {
            $($name,)*
        }

        impl Instruction {
            pub fn opcode(&self) -> Opcode {
                match self {
                    $(Self::$name { .. }=> Opcode::$name,)*
                }
            }
        }
    };
}

opcode! {
    Unreachable = 0,
}
