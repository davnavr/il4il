//! Provides the [`Resolver`] trait for resolving module imports.

use crate::runtime;
use il4il::identifier::Identifier;

/// Error type returned by [`Resolver`] methods.
pub type ResolverError = Box<dyn std::error::Error + Send + Sync>;

/// Result type returned by [`Resolver`] methods.
pub type Result<T> = std::result::Result<T, ResolverError>;

pub type FunctionImport<'env> = &'env il4il_loader::function::template::Import<'env>;

pub trait Resolver {
    fn resolve_function_import<'env>(
        runtime: &'env runtime::Runtime<'env>,
        import: FunctionImport<'env>,
    ) -> Result<runtime::Function<'env>>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ImportKind {
    Function(Identifier),
}

impl<'env> From<FunctionImport<'env>> for ImportKind {
    fn from(import: FunctionImport<'env>) -> Self {
        Self::Function(Identifier::from_id(import.symbol()))
    }
}

impl std::fmt::Display for ImportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (import_kind, import_name) = match self {
            Self::Function(id) => ("function", id),
        };

        write!(f, "{import_kind} {import_name:?}")
    }
}

struct ImportErrorInner {
    importing_module: Option<Identifier>,
    imported_module: Identifier,
    import_kind: ImportKind,
    error: ResolverError,
}

/// Error type used when resolving a reference to an import fails.
pub struct ImportError(Box<ImportErrorInner>);

impl ImportError {
    pub(crate) fn new<'env, K: Into<ImportKind>>(
        imported_module: &'env il4il_loader::module::Import<'env>,
        import_kind: K,
        error: ResolverError,
    ) -> Self {
        Self(Box::new(ImportErrorInner {
            importing_module: imported_module.importer().name().map(Identifier::from_id),
            imported_module: Identifier::from_id(imported_module.name()),
            import_kind: import_kind.into(),
            error,
        }))
    }

    pub fn error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self.0.error.as_ref()
    }
}

impl std::fmt::Debug for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImportError")
            .field("importing_module", &self.0.importing_module)
            .field("imported_module", &self.0.imported_module)
            .field("import_kind", &self.0.import_kind)
            .field("error", &self.0.error)
            .finish()
    }
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error importing {} from module {:?} ",
            self.0.import_kind, self.0.imported_module
        )?;
        if let Some(importer) = &self.0.importing_module {
            write!(f, "in module {importer:?} ")?;
        }
        write!(f, ": {}", self.0.error)
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error())
    }
}
