use crate::binary::section::Section;
use crate::versioning::SupportedFormat;

// An in-memory representation of an IL4IL module.
#[derive(Clone, Debug)]
pub struct Module {
    format_version: SupportedFormat,
    sections: Vec<Section>,
}
