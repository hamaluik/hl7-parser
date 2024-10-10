use std::collections::HashMap;
use super::FieldBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentBuilder {
    name: String,
    fields: HashMap<usize, FieldBuilder>,
}

