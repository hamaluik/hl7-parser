use crate::message::{Repeat, Separators};
use std::{collections::HashMap, fmt::Display};

use super::ComponentBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RepeatBuilder {
    Value(String),
    Components(HashMap<usize, ComponentBuilder>),
}

impl Default for RepeatBuilder {
    fn default() -> Self {
        RepeatBuilder::Value(String::new())
    }
}

impl RepeatBuilder {
    pub fn with_value(value: String) -> Self {
        RepeatBuilder::Value(value)
    }

    pub fn with_components(components: HashMap<usize, ComponentBuilder>) -> Self {
        RepeatBuilder::Components(components)
    }

    pub fn value(&self) -> Option<&String> {
        match self {
            RepeatBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn components(&self) -> Option<&HashMap<usize, ComponentBuilder>> {
        match self {
            RepeatBuilder::Components(components) => Some(components),
            _ => None,
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut String> {
        match self {
            RepeatBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn components_mut(&mut self) -> Option<&mut HashMap<usize, ComponentBuilder>> {
        match self {
            RepeatBuilder::Components(components) => Some(components),
            _ => None,
        }
    }

    pub fn has_components(&self) -> bool {
        matches!(self, RepeatBuilder::Components(_))
    }

    pub fn into_value(self) -> Option<String> {
        match self {
            RepeatBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_components(self) -> Option<HashMap<usize, ComponentBuilder>> {
        match self {
            RepeatBuilder::Components(components) => Some(components),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            RepeatBuilder::Value(value) => value.is_empty(),
            RepeatBuilder::Components(components) => components.is_empty(),
        }
    }

    pub fn set_value<S: ToString>(&mut self, value: S) {
        *self = RepeatBuilder::Value(value.to_string());
    }

    pub fn set_components(&mut self, components: HashMap<usize, ComponentBuilder>) {
        *self = RepeatBuilder::Components(components);
    }

    pub fn set_component(&mut self, index: usize, component: ComponentBuilder) {
        debug_assert!(index > 0, "Component numbers are 1-based");
        match self {
            RepeatBuilder::Components(components) => {
                components.insert(index, component);
            }
            _ => {
                let mut components = HashMap::new();
                components.insert(index, component);
                *self = RepeatBuilder::Components(components);
            }
        }
    }

    pub fn set_component_value<S: ToString>(&mut self, index: usize, value: S) {
        debug_assert!(index > 0, "Component numbers are 1-based");
        match self {
            RepeatBuilder::Components(components) => {
                let component = components
                    .entry(index)
                    .or_insert(ComponentBuilder::default());
                component.set_value(value);
            }
            _ => {
                let mut components = HashMap::new();
                let component = ComponentBuilder::Value(value.to_string());
                components.insert(index, component);
                *self = RepeatBuilder::Components(components);
            }
        }
    }

    pub fn clear(&mut self) {
        *self = RepeatBuilder::Value(String::new());
    }

    pub fn component(&self, index: usize) -> Option<&ComponentBuilder> {
        debug_assert!(index > 0, "Component numbers are 1-based");
        match self {
            RepeatBuilder::Components(components) => components.get(&index),
            _ => None,
        }
    }

    pub fn component_mut(&mut self, index: usize) -> Option<&mut ComponentBuilder> {
        debug_assert!(index > 0, "Component numbers are 1-based");
        match self {
            RepeatBuilder::Components(components) => components.get_mut(&index),
            _ => None,
        }
    }

    pub fn remove_component(&mut self, index: usize) -> Option<ComponentBuilder> {
        debug_assert!(index > 0, "Component numbers are 1-based");
        match self {
            RepeatBuilder::Components(components) => components.remove(&index),
            _ => None,
        }
    }

    pub fn display<'a>(&'a self, separators: &'a Separators) -> RepeatBuilderDisplay<'a> {
        RepeatBuilderDisplay {
            repeat: self,
            separators,
        }
    }
}

pub struct RepeatBuilderDisplay<'a> {
    repeat: &'a RepeatBuilder,
    separators: &'a Separators,
}

impl<'a> Display for RepeatBuilderDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.repeat {
            RepeatBuilder::Value(value) => self.separators.encode(value).fmt(f),
            RepeatBuilder::Components(components) => {
                if components.is_empty() {
                    return Ok(());
                }
                let max_index = components.keys().max().unwrap();
                for i in 1..=*max_index {
                    if let Some(component) = components.get(&i) {
                        write!(f, "{}", component.display(self.separators))?;
                    }
                    if i < *max_index {
                        write!(f, "{}", self.separators.component)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl<S: ToString> From<S> for RepeatBuilder {
    fn from(value: S) -> Self {
        RepeatBuilder::Value(value.to_string())
    }
}

impl<'m> From<&'m Repeat<'m>> for RepeatBuilder {
    fn from(repeat: &'m Repeat<'m>) -> Self {
        if repeat.has_components() {
            let components = repeat
                .components
                .iter()
                .enumerate()
                .map(|(i, c)| (i + 1, c.into()))
                .collect();
            RepeatBuilder::Components(components)
        } else {
            RepeatBuilder::Value(repeat.raw_value().to_string())
        }
    }
}
