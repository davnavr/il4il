//! Provides the [`Function`] struct.

use crate::interpreter::Value;
use crate::loader;
use crate::runtime::Runtime;
use std::fmt::{Debug, Formatter};

/// The result of invoking a [`HostFunction`].
pub type HostFunctionResult = Result<Box<[Value]>, Box<dyn std::error::Error + Send + Sync>>;

/// A function implemented by the host that can be imported and called by an IL4IL function.
pub struct HostFunction<'env> {
    //signature:
    closure: Box<dyn Fn(Box<[Value]>, &'env Runtime<'env>) -> HostFunctionResult + Send + Sync>,
}

impl<'env> HostFunction<'env> {
    pub fn invoke<A: Into<Box<[Value]>>>(&self, arguments: A, runtime: &'env Runtime<'env>) -> HostFunctionResult {
        (self.closure)(arguments.into(), runtime)
    }
}

impl Debug for HostFunction<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[repr(transparent)]
        pub struct AddressDebug<'a, T: ?Sized>(&'a T);

        impl<T: ?Sized> Debug for AddressDebug<'_, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:p}", self.0)
            }
        }

        f.debug_tuple("HostFunction").field(&AddressDebug(self.closure.as_ref())).finish()
    }
}

/// Indicates the implementation of an IL4IL function.
#[derive(Debug)]
pub enum Function<'env> {
    /// A function implemented by the host.
    Host(HostFunction<'env>),
    /// A function implemented in IL4IL bytecode.
    Defined(&'env loader::function::Instantiation<'env>), // TODO: How to ensure HostFunction is used when template is a function import
}

impl<'env> Function<'env> {
    //pub fn host_with_closure<F>()

    //pub fn signature(&self)
}

impl<'env> From<HostFunction<'env>> for Function<'env> {
    fn from(host_function: HostFunction<'env>) -> Self {
        Self::Host(host_function)
    }
}
