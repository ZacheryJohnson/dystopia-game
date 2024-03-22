use crate::stat::stat::Stat;

pub trait StatSource {
    fn stats(&self) -> Vec<Stat>;
}