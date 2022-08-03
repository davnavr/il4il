use crate::section::Section;

// An in-memory representation of an IL4IL module.
#[derive(Clone, Debug)]
pub struct Module {
    sections: Vec<Section>,
}
