pub use dyn_clone;

#[derive(Clone, Debug, Default)]
pub enum SatisfiableField<
    ValueT: Clone + PartialEq + PartialOrd
> {
    /// The concrete value may have any value for this field.
    /// Ignored fields will always pass satisfiability tests.
    #[default]
    Ignore,

    /// The concrete value must be the exact value of type T
    Exactly(ValueT),

    /// The concrete value must NOT be the exact value of type T
    NotExactly(ValueT),

    /// The concrete value must be in the range of values of type T
    In(Box<[ValueT]>),

    /// The concrete value must NOT be in the range of values of type T
    NotIn(Box<[ValueT]>),

    /// The concrete value must be strictly greater than the value of type T
    GreaterThan(ValueT),

    /// The concrete value must be greater than or equal to the value of type T
    GreaterThanOrEqual(ValueT),

    /// The concrete value must be strictly less than the value of type T
    LessThan(ValueT),

    /// The concrete value must be less than or equal to the value of type T
    LessThanOrEqual(ValueT),
}

impl<
    ValueT: Clone + PartialEq + PartialOrd,
> SatisfiableField<ValueT> {
    pub fn satisfied_by(&self, value: &ValueT) -> bool {
        match self {
            SatisfiableField::Ignore => true,
            SatisfiableField::Exactly(self_val) => self_val == value,
            SatisfiableField::NotExactly(self_val) => self_val != value,
            SatisfiableField::In(self_iter) => self_iter.contains(value),
            SatisfiableField::NotIn(self_iter) => !self_iter.contains(value),
            SatisfiableField::GreaterThan(self_val) => self_val > value,
            SatisfiableField::GreaterThanOrEqual(self_val) => self_val >= value,
            SatisfiableField::LessThan(self_val) => self_val < value,
            SatisfiableField::LessThanOrEqual(self_val) => self_val <= value,
        }
    }
}

pub trait SatisfiabilityTest: dyn_clone::DynClone {
    type ConcreteT;
    fn is_same_variant(&self, concrete: &Self::ConcreteT) -> bool;
    fn satisfied_by(&self, concrete: Self::ConcreteT) -> bool;
}