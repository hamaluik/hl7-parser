use crate::message::{Component, Field, Repeat, Segment, Separators, Subcomponent};

pub(crate) type Span<'m> = nom_locate::LocatedSpan<&'m str>;

mod subcomponent;
mod component;
mod repeat;
mod field;
mod segment;
mod msh;
pub(crate) mod message;

/// Parse a subcomponent using the default separators.
pub fn parse_subcomponent<'m>(input: &'m str) -> Result<Subcomponent<'m>, String> {
    let separators = Separators::default();
    subcomponent::subcomponent(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse subcomponent: {:?}", e))
}

/// Parse a subcomponent using the given separators.
pub fn parse_subcomponent_with_separators<'m>(
    input: &'m str,
    separators: Separators,
) -> Result<Subcomponent<'m>, String> {
    subcomponent::subcomponent(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse subcomponent: {:?}", e))
}

/// Parse a component using the default separators.
pub fn parse_component<'m>(input: &'m str) -> Result<Component<'m>, String> {
    let separators = Separators::default();
    component::component(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse component: {:?}", e))
}

/// Parse a component using the given separators.
pub fn parse_component_with_separators<'m>(
    input: &'m str,
    separators: Separators,
) -> Result<Component<'m>, String> {
    component::component(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse component: {:?}", e))
}

/// Parse a repeat using the default separators.
pub fn parse_repeat<'m>(input: &'m str) -> Result<Repeat<'m>, String> {
    let separators = Separators::default();
    repeat::repeat(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse repeat: {:?}", e))
}

/// Parse a repeat using the given separators.
pub fn parse_repeat_with_separators<'m>(
    input: &'m str,
    separators: Separators,
) -> Result<Repeat<'m>, String> {
    repeat::repeat(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse repeat: {:?}", e))
}

/// Parse a field using the default separators.
pub fn parse_field<'m>(input: &'m str) -> Result<Field<'m>, String> {
    let separators = Separators::default();
    field::field(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse field: {:?}", e))
}

/// Parse a field using the given separators.
pub fn parse_field_with_separators<'m>(
    input: &'m str,
    separators: Separators,
) -> Result<Field<'m>, String> {
    field::field(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse field: {:?}", e))
}

/// Parse a segment using the default separators.
pub fn parse_segment<'m>(input: &'m str) -> Result<Segment<'m>, String> {
    let separators = Separators::default();
    segment::segment(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse segment: {:?}", e))
}

/// Parse a segment using the given separators.
pub fn parse_segment_with_separators<'m>(
    input: &'m str,
    separators: Separators,
) -> Result<Segment<'m>, String> {
    segment::segment(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse segment: {:?}", e))
}

/// Parse a MSH segment and return the separators and the segment.
pub fn parse_msh<'m>(input: &'m str) -> Result<(Separators, Segment<'m>), String> {
    msh::msh()(Span::new(input))
        .map(|(_, m)| (m.separators.clone(), m.into()))
        .map_err(|e| format!("Failed to parse MSH segment: {:?}", e))
}

/// Parse a complete HL7 message.
pub fn parse_message<'m>(input: &'m str) -> Result<crate::Message<'m>, String> {
    crate::parser::message::message()(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| format!("Failed to parse message: {:?}", e))
}

