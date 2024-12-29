use serde::{Deserialize, Serialize};
use crate::attribute::attribute_type::AttributeType;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AttributeInstance(AttributeType, f32);

impl AttributeInstance {
    pub fn new(attribute_type: AttributeType, value: f32) -> Self { Self(attribute_type, value) }
    pub fn attribute_type(&self) -> &AttributeType { &self.0 }
    pub fn value(&self) -> f32 { self.1 }
}

impl From<AttributeInstance> for (AttributeType, f32) {
    fn from(val: AttributeInstance) -> Self {
        (val.0, val.1)
    }
}

impl Into<AttributeType> for AttributeInstance {
    fn into(self) -> AttributeType { self.0 }
}

impl Into<f32> for AttributeInstance {
    fn into(self) -> f32 { self.1 }
}