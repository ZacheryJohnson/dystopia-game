use std::fmt::Formatter;
use std::rc::Rc;
pub use dyn_clone;
pub use ahash;

#[derive(Clone, Default)]
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
    In(Vec<ValueT>),

    /// The concrete value must NOT be in the range of values of type T
    NotIn(Vec<ValueT>),

    /// The concrete value must be strictly greater than the value of type T
    GreaterThan(ValueT),

    /// The concrete value must be greater than or equal to the value of type T
    GreaterThanOrEqual(ValueT),

    /// The concrete value must be strictly less than the value of type T
    LessThan(ValueT),

    /// The concrete value must be less than or equal to the value of type T
    LessThanOrEqual(ValueT),

    /// The concrete value must pass a provided lambda
    Lambda(Rc<dyn Fn(ValueT) -> bool>)
}

impl<ValueT: Clone + PartialEq + PartialOrd> std::fmt::Debug for SatisfiableField<ValueT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SatisfiableField::Lambda(_) => write!(f, "<lambda fn>"),
            _ => write!(f, "{:?}", self)
        }
    }
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
            SatisfiableField::Lambda(lambda_fn) => lambda_fn(value.to_owned())
        }
    }
}

pub trait SatisfiabilityTest: dyn_clone::DynClone {
    type ConcreteT;
    /// Is this variant the same as the other variant?
    fn is_same_variant(&self, concrete: &Self::ConcreteT) -> bool;

    /// Can this test be satisfied by the concrete value?
    fn satisfied_by(&self, concrete: Self::ConcreteT) -> bool;
}

pub trait Uniqueness: dyn_clone::DynClone {
    fn unique_key(&self) -> u64;

    fn has_same_unique_key(&self, other: &Self) -> bool {
        if self.unique_key() == 0 || other.unique_key() == 0 {
            return false;
        }

        self.unique_key() == other.unique_key()
    }
}