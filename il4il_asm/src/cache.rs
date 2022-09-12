//! Module for caching data.

/// Provides ownership of strings.
pub struct StringCache<'a> {
    lookup: std::cell::RefCell<rustc_hash::FxHashMap<&'a str, ()>>,
    strings: typed_arena::Arena<u8>,
}

impl<'a> StringCache<'a> {
    pub fn new() -> Self {
        Self {
            lookup: Default::default(),
            strings: Default::default(),
        }
    }

    pub(crate) fn get_or_insert(&'a self, buffer: &mut String) -> &'a str {
        let entry = match buffer.as_str() {
            "format" => "format",
            "metadata" => "metadata",
            s => {
                let mut lookup = self.lookup.borrow_mut();
                if let Some((entry, _)) = lookup.get_key_value(s) {
                    *entry
                } else {
                    let entry: &'a str = self.strings.alloc_str(s);
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
