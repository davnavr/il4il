//! Module for caching data.

use std::borrow::Cow;
use std::ops::Deref;

pub trait StringRef<'a>: Deref<Target = str> + std::fmt::Display + std::fmt::Debug + 'a {
    fn into_cow(self) -> Cow<'a, str>;
}

impl<'cache> StringRef<'cache> for &'cache str {
    fn into_cow(self) -> Cow<'cache, str> {
        Cow::Borrowed(self)
    }
}

impl<'a> StringRef<'a> for std::rc::Rc<str> {
    fn into_cow(self) -> Cow<'a, str> {
        Cow::Owned(String::from(self.deref()))
    }
}

/// Trait implemented for string caches.
pub trait StringCache<'this, 'str: 'this> {
    type Ref: StringRef<'str>;

    /// Allocates a string without caching it.
    fn store(&'this self, buffer: &mut String) -> Self::Ref;

    /// Allocates a string, or retrieves an equivalent cached string.
    fn get_or_store(&'this self, buffer: &mut String) -> Self::Ref {
        Self::store(self, buffer)
    }
}

type HashSetCell<T> = std::cell::RefCell<rustc_hash::FxHashMap<T, ()>>;

/// Provides ownership of strings.
///
/// In cases where self-recursion cannot be used, use [`RcStringCache`] instead.
pub struct StringArena<'cache> {
    lookup: HashSetCell<&'cache str>,
    strings: typed_arena::Arena<u8>,
}

impl<'cache> StringArena<'cache> {
    pub fn new() -> Self {
        Self {
            lookup: Default::default(),
            strings: Default::default(),
        }
    }
}

impl Default for StringArena<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StringArena<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StringCache").field(&self.lookup).finish()
    }
}

impl<'cache> StringCache<'cache, 'cache> for StringArena<'cache> {
    type Ref = &'cache str;

    fn store(&'cache self, buffer: &mut String) -> Self::Ref {
        if buffer.is_empty() {
            return Default::default();
        }

        let entry = self.strings.alloc_str(buffer.as_str());
        buffer.clear();
        entry
    }

    fn get_or_store(&'cache self, buffer: &mut String) -> Self::Ref {
        if buffer.is_empty() {
            return Default::default();
        }

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

#[derive(Debug, Default)]
pub struct RcStringCache {
    cached: HashSetCell<std::rc::Rc<str>>,
    others: std::cell::RefCell<Vec<std::rc::Weak<str>>>,
}

impl RcStringCache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'this> StringCache<'this, 'static> for RcStringCache {
    type Ref = std::rc::Rc<str>;

    fn store(&'this self, buffer: &mut String) -> Self::Ref {
        if buffer.is_empty() {
            return std::rc::Rc::from(Box::default());
        }

        let stored = std::rc::Rc::<str>::from(buffer.as_str());
        buffer.clear();
        self.others.borrow_mut().push(std::rc::Rc::downgrade(&stored));
        stored
    }

    fn get_or_store(&'this self, buffer: &mut String) -> Self::Ref {
        if buffer.is_empty() {
            return std::rc::Rc::from(Box::default());
        }

        let mut lookup = self.cached.borrow_mut();
        let entry = if let Some((existing, _)) = lookup.get_key_value(buffer.as_str()) {
            existing.clone()
        } else {
            let entry = std::rc::Rc::<str>::from(buffer.as_str());
            lookup.insert(entry.clone(), ());
            entry
        };

        buffer.clear();
        entry
    }
}
