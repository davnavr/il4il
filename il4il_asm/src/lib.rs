//! The IL4IL bytecode assembler.

pub mod assembler;
pub mod cache;
pub mod error;
pub mod input;
pub mod lexer;
pub mod location;
pub mod parser;
pub mod syntax;

pub use il4il as bytecode;

/// Error type used by the top-level [`assemble`] function.
#[derive(Debug)]
pub enum FullError<E> {
    /// Indicates that an error originated from the input.
    InvalidInput(E),
    /// Indicates that assembly failed with one or more errors.
    AssemblyFailed(Vec<error::Error>),
}

impl FullError<std::convert::Infallible> {
    pub fn into_assembly_error(self) -> Vec<error::Error> {
        match self {
            Self::AssemblyFailed(errors) => errors,
            Self::InvalidInput(_) => unreachable!(),
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for FullError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(e) => write!(f, "encounted invalid input: {e}"),
            Self::AssemblyFailed(errors) => errors.iter().try_for_each(|e| writeln!(f, "{e}")),
        }
    }
}

impl<E: std::error::Error> std::error::Error for FullError<E> {}

/// Given IL4IL assembly, produces an IL4IL module.
///
/// # Errors
///
/// Any errors encountered during assembly are collected and returned.
pub fn assemble<'cache, I: input::IntoInput>(
    input: I,
    string_cache: &'cache cache::StringCache<'cache>,
) -> Result<assembler::Output<'cache>, FullError<<I::Source as input::Input>::Error>> {
    let mut errors = Vec::new();
    let tokens = lexer::tokenize(input, string_cache).map_err(FullError::InvalidInput)?;
    let tree = parser::parse(tokens, &mut errors);
    let output = assembler::assemble(tree, &mut errors);
    if errors.is_empty() {
        Ok(output)
    } else {
        Err(FullError::AssemblyFailed(errors))
    }
}
