//! Module for manipulating debugger breakpoints.

use crate::loader;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

//pub enum Condition

///// Represents a set of conditions indicating whether a breakpoint should be hit.
//pub struct ConditionSet {
//  functions: ,
//  hit_count: std::ops::RangeInclusive,
//}

/// Represents the location of a breakpoint.
#[derive(Clone, Copy)]
pub struct Location<'env> {
    block: &'env loader::code::Block<'env>,
    instruction: usize,
}

impl<'env> Location<'env> {
    pub fn new(block: &'env loader::code::Block<'env>, instruction: usize) -> Self {
        Self { block, instruction }
    }

    pub fn block(&self) -> &'env loader::code::Block<'env> {
        self.block
    }

    pub fn instruction(&self) -> usize {
        self.instruction
    }
}

impl Debug for Location<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Location")
            .field("block", &self.block.index())
            .field("instruction", &self.instruction)
            .finish()
    }
}

impl PartialEq for Location<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.block, other.block) && self.instruction == other.instruction
    }
}

impl Eq for Location<'_> {}

impl std::hash::Hash for Location<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.block as *const _ as usize);
        state.write_usize(self.instruction);
    }
}

// Safety: Above Eq and Hash implementations are deterministic
unsafe impl flashmap::TrustedHashEq for Location<'_> {}

//pub struct BreakpointConditionsInner { functions: rustc_hash::FxHashMap<_, ()> }

//pub struct BreakpointConditions(Mutex<BreakpointConditionsInner>);

/// Represents a debugger breakpoint.
#[derive(Debug)]
pub struct Breakpoint<'env> {
    disabled: AtomicBool,
    hit_count: AtomicU64,
    //conditions: BreakpointConditions,
    _phantom: std::marker::PhantomData<&'env ()>,
}

impl<'env> Breakpoint<'env> {
    fn new() -> Self {
        Self {
            disabled: AtomicBool::new(false),
            hit_count: AtomicU64::new(0),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled.load(Ordering::Acquire)
    }

    pub fn hit_count(&self) -> u64 {
        self.hit_count.load(Ordering::Acquire)
    }
}

type BuildHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;

type WriteHandle<'env> = flashmap::WriteHandle<Location<'env>, Arc<Breakpoint<'env>>, BuildHasher>;

type ReadHandle<'env> = flashmap::ReadHandle<Location<'env>, Arc<Breakpoint<'env>>, BuildHasher>;

/// Represents a set of breakpoints that indicate where IL4IL bytecode interpretation should stop.
pub struct BreakpointLookup<'env> {
    initialized: Arc<AtomicBool>,
    lookup: ReadHandle<'env>,
}

impl<'env> BreakpointLookup<'env> {
    pub fn is_empty(&self) -> bool {
        !self.initialized.load(Ordering::Acquire) || self.lookup.guard().is_empty()
    }

    pub fn get<L: ?Sized>(&self, location: &L) -> Option<Arc<Breakpoint<'env>>>
    where
        Location<'env>: std::borrow::Borrow<L>,
        L: std::hash::Hash + Eq,
    {
        self.lookup.guard().get(location).cloned()
    }
}

/// Allows insertion and removal of breakpoints from a [`BreakpointLookup`].
pub struct BreakpointWriter<'env> {
    initialized: Arc<AtomicBool>,
    handle: std::cell::RefCell<WriteHandle<'env>>,
}

impl<'env> BreakpointWriter<'env> {
    pub fn insert(&self, location: Location<'env>) {
        self.initialized.store(true, Ordering::Release);
        self.handle.borrow_mut().guard().insert(location, Arc::new(Breakpoint::new()));
    }

    //pub fn insert_with_conditions<F: &mut BreakpointConditions>(&self, location: Location<'env>, conditions: F)
}
