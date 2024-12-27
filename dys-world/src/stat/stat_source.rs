use crate::stat::definition::StatDefinition;

pub trait StatSource {
    fn stats(&self) -> Vec<StatDefinition>;
}