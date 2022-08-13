//! Functions for manipulating IL4IL module metadata.

use crate::error::{self, Message};
use crate::identifier::Identifier;
use crate::pointer;
use il4il::binary::section::Metadata;
use std::borrow::Cow;

pub type Builder = Vec<Metadata<'static>>;

/// Adds a module name to this metadata section, copying the contents of an identifier string.
///
/// # Safety
///
/// Callers should ensure that the metadata and name have not been disposed.
///
/// # Panics
///
/// Panics if the [`error` pointer is not valid](#error::catch).
#[no_mangle]
pub unsafe extern "C" fn il4il_metadata_section_add_name(metadata: *mut Builder, name: *mut Identifier, error: *mut *mut Message) {
    let f = || -> Result<_, Message> {
        let builder_mut: &mut Builder;
        let name_ref: &Identifier;

        unsafe {
            // Safety: metadata is assumed to dereferenceable
            builder_mut = pointer::as_mut("metadata", metadata)?;
            name_ref = pointer::as_mut("name", name)? as &Identifier;
        }

        builder_mut.push(Metadata::Name(Cow::Owned(name_ref.clone())));
        Ok(())
    };

    unsafe {
        // Safety: error is assumed to be dereferenceable.
        error::catch_or_default(f, error)
    }
}
