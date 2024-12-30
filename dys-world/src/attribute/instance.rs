use serde::{Deserialize, Serialize};
use crate::attribute::attribute_type::AttributeType;

pub type AttributeValueT = f32;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AttributeInstance(AttributeType, AttributeValueT);

impl AttributeInstance {
    pub fn new(attribute_type: AttributeType, value: AttributeValueT) -> Self { Self(attribute_type, value) }
    pub fn attribute_type(&self) -> &AttributeType { &self.0 }
    pub fn value(&self) -> AttributeValueT { self.1 }
}

impl From<AttributeInstance> for (AttributeType, AttributeValueT) {
    fn from(val: AttributeInstance) -> Self {
        (val.0, val.1)
    }
}

impl Into<AttributeType> for AttributeInstance {
    fn into(self) -> AttributeType { self.0 }
}

impl Into<f32> for AttributeInstance {
    fn into(self) -> AttributeValueT { self.1 }
}