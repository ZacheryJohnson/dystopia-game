use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::attribute::attribute_type::AttributeType;

pub type AttributeValueT = f32;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, TS)]
#[ts(export)]
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

impl From<AttributeInstance> for AttributeType {
    fn from(value: AttributeInstance) -> Self {
        value.0.clone()
    }
}

impl From<AttributeInstance> for f32 {
    fn from(value: AttributeInstance) -> Self {
        value.1
    }
}