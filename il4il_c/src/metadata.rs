//! Functions for manipulating IL4IL module metadata.

use crate::identifier::Identifier;
use crate::pointer::Exposed;
use il4il::module::section::MetadataKind;

pub type Metadata = il4il::module::section::Metadata<'static>;

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
pub unsafe extern "C" fn il4il_metadata_kind<'a>(metadata: Exposed<'a, &'a Metadata>) -> u8 {
    let r = unsafe {
        // Safety: Caller ensures metadata pointer is valid
        metadata.unwrap().expect("metadata")
    };

    let kind = match r {
        Metadata::Name(_) => MetadataKind::Name,
    };

    u8::from(kind)
}

/// Allocates an identifier string from the module name metadata, or `null` if the metadata is not a module name.
///
/// If successful, the returned string should be disposed of later with
/// [`il4il_identifier_dispose`](crate::identifier::il4il_identifier_dispose).
///
/// # Safety
///
/// Callers must ensure that the underlying browser has not been disposed.
///
/// # Panics
///
/// Panics if the [`metadata` pointer is not valid](crate::pointer).
#[no_mangle]
pub unsafe extern "C" fn il4il_metadata_module_name<'a>(metadata: Exposed<'a, &'a Metadata>) -> *mut Identifier {
    let r = unsafe {
        // Safety: Caller ensures metadata pointer is valid
        metadata.unwrap().expect("metadata")
    };

    match r {
        Metadata::Name(name) => Box::into_raw(Box::new(name.clone().name.into_owned())),
        //_ => std::ptr::null_mut(),
    }
}
