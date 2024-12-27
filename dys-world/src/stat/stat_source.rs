use crate::stat::instance::StatInstance;

pub trait StatSource {
    fn stats(&self) -> Vec<StatInstance>;
}