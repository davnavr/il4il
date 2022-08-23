//! Helper module to help with [`Debug`] representations.

use std::fmt::{Debug, Formatter};

#[repr(transparent)]
pub struct LazyDebug<'a, T, U>(pub &'a lazy_init::LazyTransform<T, U>);

impl<T, U: Debug> Debug for LazyDebug<'_, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0.get(), f)
    }
}
