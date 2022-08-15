//! Provides functions to examine validated IL4IL modules.

use il4il::validation;

pub type Instance = validation::ValidModule<'static>;
