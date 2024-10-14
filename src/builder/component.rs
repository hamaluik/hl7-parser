use std::{collections::HashMap, fmt::Display};

use display::ComponentBuilderDisplay;

use crate::{message::{Component, Separators}, timestamps::TimeStamp};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComponentBuilder {
    Value(String),
    Subcomponents(HashMap<usize, String>),
}

impl Default for ComponentBuilder {
    fn default() -> Self {
        ComponentBuilder::Value(String::new())
    }
}

impl ComponentBuilder {
    pub fn with_value(value: String) -> Self {
        ComponentBuilder::Value(value)
    }

    pub fn with_subcomponents(subcomponents: HashMap<usize, String>) -> Self {
        ComponentBuilder::Subcomponents(subcomponents)
    }

    pub fn value(&self) -> Option<&String> {
        match self {
            ComponentBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn subcomponents(&self) -> Option<&HashMap<usize, String>> {
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => Some(subcomponents),
            _ => None,
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut String> {
        match self {
            ComponentBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn subcomponents_mut(&mut self) -> Option<&mut HashMap<usize, String>> {
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => Some(subcomponents),
            _ => None,
        }
    }

    pub fn has_subcomponents(&self) -> bool {
        matches!(self, ComponentBuilder::Subcomponents(_))
    }

    pub fn into_value(self) -> Option<String> {
        match self {
            ComponentBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_subcomponents(self) -> Option<HashMap<usize, String>> {
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => Some(subcomponents),
            _ => None,
        }
    }

    pub fn set_value<S: ToString>(&mut self, value: S) {
        *self = ComponentBuilder::Value(value.to_string());
    }

    pub fn set_timestamp<T: Into<TimeStamp>>(&mut self, timestamp: T) {
        *self = ComponentBuilder::Value(timestamp.into().to_string());
    }

    pub fn set_subcomponents(&mut self, subcomponents: HashMap<usize, String>) {
        *self = ComponentBuilder::Subcomponents(subcomponents);
    }

    pub fn set_subcomponent<S: ToString>(&mut self, index: usize, value: S) {
        debug_assert!(index > 0, "Subcomponent index must be greater than 0");
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => {
                subcomponents.insert(index, value.to_string());
            }
            _ => {
                let mut subcomponents = HashMap::new();
                subcomponents.insert(index, value.to_string());
                *self = ComponentBuilder::Subcomponents(subcomponents);
            }
        }
    }

    pub fn with_subcomponent<S: ToString>(mut self, index: usize, value: S) -> Self {
        self.set_subcomponent(index, value);
        self
    }

    pub fn set_subcomponent_timestamp<T: Into<TimeStamp>>(&mut self, index: usize, timestamp: T,) {
        self.set_subcomponent(index, timestamp.into().to_string());
    }

    pub fn with_subcomponent_timestamp<T: Into<TimeStamp>>(mut self, index: usize, timestamp: T) -> Self {
        self.set_subcomponent_timestamp(index, timestamp);
        self
    }

    pub fn subcomponent(&self, index: usize) -> Option<&String> {
        debug_assert!(index > 0, "Subcomponent index must be greater than 0");
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => subcomponents.get(&index),
            _ => None,
        }
    }

    pub fn subcomponent_mut(&mut self, index: usize) -> Option<&mut String> {
        debug_assert!(index > 0, "Subcomponent index must be greater than 0");
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => subcomponents.get_mut(&index),
            _ => None,
        }
    }

    pub fn remove_subcomponent(&mut self, index: usize) -> Option<String> {
        debug_assert!(index > 0, "Subcomponent index must be greater than 0");
        match self {
            ComponentBuilder::Subcomponents(subcomponents) => subcomponents.remove(&index),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        *self = ComponentBuilder::Value(String::new());
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ComponentBuilder::Value(value) => value.is_empty(),
            ComponentBuilder::Subcomponents(subcomponents) => subcomponents.is_empty(),
        }
    }

    pub fn display<'a>(&'a self, separators: &'a Separators) -> ComponentBuilderDisplay<'a> {
        ComponentBuilderDisplay {
            component: self,
            separators,
        }
    }
}

mod display {
    use super::*;

    pub struct ComponentBuilderDisplay<'a> {
        pub(super) component: &'a ComponentBuilder,
        pub(super) separators: &'a Separators,
    }

    impl Display for ComponentBuilderDisplay<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self.component {
                ComponentBuilder::Value(value) => self.separators.encode(value).fmt(f),
                ComponentBuilder::Subcomponents(subcomponents) => {
                    if subcomponents.is_empty() {
                        return Ok(());
                    }
                    let max_index = subcomponents.keys().max().unwrap();
                    for i in 1..=*max_index {
                        if let Some(value) = subcomponents.get(&i) {
                            self.separators.encode(value).fmt(f)?;
                        }
                        if i < *max_index {
                            write!(f, "{}", self.separators.subcomponent)?;
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

impl<S: ToString> From<S> for ComponentBuilder {
    fn from(value: S) -> Self {
        ComponentBuilder::Value(value.to_string())
    }
}

impl<'m> From<&'m Component<'m>> for ComponentBuilder {
    fn from(component: &'m Component<'m>) -> Self {
        if component.subcomponents.len() <= 1 {
            ComponentBuilder::Value(component.source.to_string())
        } else {
            let subcomponents = component
                .subcomponents
                .iter()
                .enumerate()
                .map(|(i, subcomponent)| (i + 1, subcomponent.value.to_string()))
                .collect();
            ComponentBuilder::Subcomponents(subcomponents)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::Subcomponent;
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_display_component_builder() {
        let separators = Separators::default();
        let component = ComponentBuilder::with_value("foo".to_string());
        let display = component.display(&separators).to_string();
        assert_eq!(display, "foo");

        let mut subcomponents = HashMap::new();
        subcomponents.insert(1, "bar".to_string());
        subcomponents.insert(3, "baz".to_string());
        let component = ComponentBuilder::with_subcomponents(subcomponents);
        let display = component.display(&separators).to_string();
        assert_eq!(display, "bar&&baz");
    }

    #[test]
    fn can_convert_component_to_component_builder() {
        let component = Component {
            source: "foo&bar",
            subcomponents: vec![
                Subcomponent {
                    value: "foo",
                    range: 0..1, // ignore
                },
                Subcomponent {
                    value: "bar",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        let component_builder: ComponentBuilder = (&component).into();
        assert_eq!(
            component_builder,
            ComponentBuilder::with_subcomponents(
                vec![(1, "foo".to_string()), (2, "bar".to_string())]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn can_convert_with_singular_value() {
        let component = crate::parser::parse_component("foo").expect("Can parse component");
        let component_builder = ComponentBuilder::from(&component);
        assert_eq!(component_builder, ComponentBuilder::with_value("foo".to_string()));
    }
}
