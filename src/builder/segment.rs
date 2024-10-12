use crate::message::Segment;
use std::{collections::HashMap, fmt::Display};

use super::FieldBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentBuilder {
    pub name: String,
    pub fields: HashMap<usize, FieldBuilder>,
}

impl Default for SegmentBuilder {
    fn default() -> Self {
        SegmentBuilder {
            name: String::new(),
            fields: HashMap::new(),
        }
    }
}

impl SegmentBuilder {
    pub fn new<S: ToString>(name: S) -> Self {
        SegmentBuilder {
            name: name.to_string(),
            fields: HashMap::new(),
        }
    }

    pub fn push_field(&mut self, field: FieldBuilder) {
        let index = self.fields.len();
        self.fields.insert(index + 1, field);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &HashMap<usize, FieldBuilder> {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut HashMap<usize, FieldBuilder> {
        &mut self.fields
    }

    pub fn field(&self, index: usize) -> Option<&FieldBuilder> {
        debug_assert!(index > 0, "Field numbers are 1-based");
        self.fields.get(&index)
    }

    pub fn field_mut(&mut self, index: usize) -> Option<&mut FieldBuilder> {
        debug_assert!(index > 0, "Field numbers are 1-based");
        self.fields.get_mut(&index)
    }

    pub fn remove_field(&mut self, index: usize) -> Option<FieldBuilder> {
        debug_assert!(index > 0, "Field numbers are 1-based");
        self.fields.remove(&index)
    }

    pub fn has_field(&self, index: usize) -> bool {
        debug_assert!(index > 0, "Field numbers are 1-based");
        self.fields.contains_key(&index)
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn clear(&mut self) {
        self.fields.clear();
    }

    pub fn set_name<S: ToString>(&mut self, name: S) {
        self.name = name.to_string();
    }

    pub fn set_field(&mut self, index: usize, field: FieldBuilder) {
        debug_assert!(index > 0, "Field numbers are 1-based");
        self.fields.insert(index, field);
    }

    pub fn set_field_value<S: ToString>(&mut self, index: usize, value: S) {
        debug_assert!(index > 0, "Field numbers are 1-based");
        let field = self.fields.entry(index).or_insert(FieldBuilder::default());
        field.set_value(value.to_string());
    }

    pub fn display<'a>(&'a self, separators: &'a super::Separators) -> SegmentBuilderDisplay<'a> {
        SegmentBuilderDisplay {
            segment: self,
            separators,
        }
    }
}

pub struct SegmentBuilderDisplay<'a> {
    segment: &'a SegmentBuilder,
    separators: &'a super::Separators,
}

impl<'a> Display for SegmentBuilderDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segment.name)?;

        if self.segment.fields.is_empty() {
            return Ok(());
        }

        let start_index = if self.segment.name == "MSH" {
            write!(f, "{}", self.separators)?;
            3
        } else {
            1
        };
        write!(f, "{}", self.separators.field)?;
        let max_index = self.segment.fields.keys().max().unwrap();
        for i in start_index..=*max_index {
            if let Some(field) = self.segment.fields.get(&i) {
                write!(f, "{}", field.display(self.separators))?;
            }
            if i < *max_index {
                write!(f, "{}", self.separators.field)?;
            }
        }
        Ok(())
    }
}

impl<'m> From<&'m Segment<'m>> for SegmentBuilder {
    fn from(segment: &'m Segment) -> Self {
        let mut builder = SegmentBuilder::new(segment.name);
        builder.fields = segment
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| (index + 1, field.into()))
            .collect();
        builder
    }
}

#[cfg(test)]
mod tests {
    use crate::message::Separators;

    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_convert_from_segment() {
        let segment = crate::parser::parse_segment(r#"PID|1|2|3"#).unwrap();
        let builder: SegmentBuilder = SegmentBuilder::from(&segment);
        assert_eq!(builder.name(), "PID");
        assert_eq!(builder.fields().len(), 3);
        assert_eq!(builder.field(1).unwrap().value().unwrap(), "1");
        assert_eq!(builder.field(2).unwrap().value().unwrap(), "2");
        assert_eq!(builder.field(3).unwrap().value().unwrap(), "3");
    }

    #[test]
    fn can_display_segment() {
        let segment = crate::parser::parse_segment(r#"PID|1|2|3"#).unwrap();
        let builder: SegmentBuilder = SegmentBuilder::from(&segment);
        let separators = Separators::default();
        let display = builder.display(&separators).to_string();
        assert_eq!(display, r#"PID|1|2|3"#);
    }
}
