//! Provides the [`Function`] struct.

use crate::interpreter::value::Value;
use crate::runtime::Runtime;
use il4il_loader::function::Instantiation;
use std::fmt::{Debug, Formatter};

/// The result of invoking a [`HostFunction`].
pub type HostFunctionResult = Result<Box<[Value]>, Box<dyn std::error::Error + Send + Sync>>;

type HostFunctionClosure = Box<dyn for<'env> Fn(&[Value], &'env Runtime<'env>) -> HostFunctionResult + Send + Sync>;

/// A function implemented by the host that can be imported and called by an IL4IL function.
pub struct HostFunction {
    //signature:
    closure: HostFunctionClosure,
}

// TODO: Maybe have a trait to allow conversion of values (e.g. u32, u64, etc.), and allow easy construction of HostFunction from closures (e.g. Fn(u32, u32) should be easily translated)

impl HostFunction {
    pub fn invoke<'env>(&self, arguments: &[Value], runtime: &'env Runtime<'env>) -> HostFunctionResult {
        (self.closure)(arguments, runtime)
    }
}

impl Debug for HostFunction {
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
pub enum FunctionImplementation<'env> {
    /// A function implemented by the host.
    Host(HostFunction),
    /// A function implemented in IL4IL bytecode.
    Defined(&'env il4il_loader::function::template::Definition<'env>),
}

impl<'env> FunctionImplementation<'env> {
    //pub fn host_with_closure<F>()

    //pub fn signature(&self)
}

impl<'env> From<HostFunction> for FunctionImplementation<'env> {
    fn from(host_function: HostFunction) -> Self {
        Self::Host(host_function)
    }
}

pub struct Function<'env> {
    module: &'env crate::runtime::Module<'env>,
    instantiation: &'env Instantiation<'env>,
    implementation: &'env FunctionImplementation<'env>,
}

impl<'env> Function<'env> {
    pub(super) fn new(
        module: &'env crate::runtime::Module<'env>,
        instantiation: &'env Instantiation<'env>,
    ) -> Result<Self, crate::runtime::resolver::ImportError> {
        Ok(Self {
            module,
            instantiation,
            implementation: module.get_function_implementation(instantiation.template().index())?,
        })
    }

    pub fn module(&self) -> &'env crate::runtime::Module<'env> {
        self.module
    }

    pub fn instantiation(&self) -> &'env Instantiation<'env> {
        self.instantiation
    }

    pub fn implementation(&self) -> &'env FunctionImplementation<'env> {
        &self.implementation
    }
}

impl Debug for Function<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("implementation", &self.implementation)
            .finish_non_exhaustive()
    }
}
