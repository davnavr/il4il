//! Functions for manipulating IL4IL module metadata.

use crate::pointer::Exposed;

pub type Metadata = il4il::binary::section::Metadata<'static>;

pub use il4il::binary::section::MetadataKind;

/// Gets the number of entries in the module's metadata sections.
///
/// # Safety
///
/// Callers must ensure that the underlying browser has not been disposed.
///
/// # Panics
///
/// Panics if the [`metadata` pointer is not valid](crate::pointer).
#[no_mangle]
pub unsafe fn il4il_metadata_kind<'a>(metadata: Exposed<'a, &'a Metadata>) -> MetadataKind {
    let r = unsafe {
        // Safety: Caller ensures metadata pointer is valid
        metadata.unwrap().expect("metadata")
    };

    match r {
        Metadata::Name(_) => MetadataKind::Name,
    }
}
