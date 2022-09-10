//! Helper module to help with [`Debug`] representations.

use std::fmt::{Debug, Formatter};

#[repr(transparent)]
pub struct LazyDebug<'a, T, U>(pub &'a lazy_init::LazyTransform<T, U>);

impl<T, U: Debug> Debug for LazyDebug<'_, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.0.get() {
            Debug::fmt(value, f)
        } else {
            f.write_str("<uninitialized>")
        }
    }
}
