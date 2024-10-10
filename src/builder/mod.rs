mod segment;
pub use segment::*;

mod field;
pub use field::*;

mod repeat;
pub use repeat::*;

mod component;
pub use component::*;

use crate::message::Separators;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageBuilder {
    separators: Separators,
    segments: Vec<SegmentBuilder>,
}

