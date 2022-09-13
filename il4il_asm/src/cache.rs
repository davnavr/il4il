//! Module for caching data.

/// Provides ownership of strings.
pub struct StringCache<'cache> {
    lookup: std::cell::RefCell<rustc_hash::FxHashMap<&'cache str, ()>>,
    strings: typed_arena::Arena<u8>,
}

impl<'cache> StringCache<'cache> {
    pub fn new() -> Self {
        Self {
            lookup: Default::default(),
            strings: Default::default(),
        }
    }

    /// Allocates a string in this cache without storing it in the lookup.
    pub(crate) fn store(&'cache self, buffer: &mut String) -> &'cache str {
        let entry = self.strings.alloc_str(buffer.as_str());
        buffer.clear();
        entry
    }

    /// Inserts a string, or retrieves an equivalent cached string.
    pub(crate) fn get_or_insert(&'cache self, buffer: &mut String) -> &'cache str {
        let entry = match buffer.as_str() {
            "format" => "format",
            "metadata" => "metadata",
            s => {
                let mut lookup = self.lookup.borrow_mut();
                if let Some((entry, _)) = lookup.get_key_value(s) {
                    *entry
                } else {
                    let entry: &'cache str = self.strings.alloc_str(s);
                    lookup.insert(entry, ());
                    entry
                }
            }
        };

        buffer.clear();
        entry
    }
}

impl Default for StringCache<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StringCache<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StringCache").field(&self.lookup).finish()
    }
}
