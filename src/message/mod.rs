mod separators;
pub use separators::*;
mod subcomponent;
pub use subcomponent::*;
mod component;
pub use component::*;
mod repeat;
pub use repeat::*;
mod field;
pub use field::*;
mod segment;
pub use segment::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    pub segments: Vec<Segment<'m>>,
    pub separators: Separators,
}

// impl<'m> Message<'m> {
//     pub fn from_str(input: &'m str) -> Result<Self, String> {
//         crate::parser::message::message()(input)
//             .map(|(_, m)| m)
//             .map_err(move |e| format!("Failed to parse message: {:?}", e))
//     }
// }
