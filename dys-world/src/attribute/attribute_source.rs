use crate::attribute::instance::AttributeInstance;

pub trait AttributeSource {
    fn stats(&self) -> Vec<AttributeInstance>;
}