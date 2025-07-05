use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;
pub use dyn_clone;
pub use ahash;

/// A SatisfiableField represents an abstract test that will be performed on a concrete value.
///
#[derive(Clone, Default)]
pub enum SatisfiableField<
    ConcreteT: Clone + PartialEq + PartialOrd
> {
    /// The concrete value may have any value for this field.
    /// Ignored fields will always pass satisfiability tests.
    /// ```
    /// # use dys_satisfiable::SatisfiableField;
    ///
    /// let ignored = SatisfiableField::<u32>::Ignore;
    /// assert!(ignored.satisfied_by(&u32::MIN));
    /// assert!(ignored.satisfied_by(&u32::MAX));
    /// ```
    #[default]
    Ignore,

    /// The concrete value must be the exact value of type ConcreteT
    /// ```
    /// # use dys_satisfiable::SatisfiableField;
    /// let exactly_three = SatisfiableField::Exactly(3u32);
    /// assert_eq!(true, exactly_three.satisfied_by(&3u32));
    /// assert_eq!(false, exactly_three.satisfied_by(&4u32));
    /// ```
    Exactly(ConcreteT),

    /// The concrete value must NOT be the exact value of type ConcreteT
    /// ```
    /// # use dys_satisfiable::SatisfiableField;
    /// let not_exactly_three = SatisfiableField::NotExactly(3u32);
    /// assert_eq!(true, not_exactly_three.satisfied_by(&4u32));
    /// assert_eq!(false, not_exactly_three.satisfied_by(&3u32));
    /// ```
    NotExactly(ConcreteT),

    /// The concrete value must be in the range of values of type ConcreteT
    /// ```
    /// # use dys_satisfiable::SatisfiableField;
    /// let allowed_values = SatisfiableField::In(vec![
    ///     1u32,
    ///     2u32,
    ///     3u32,
    /// ]);
    ///
    /// assert_eq!(false, allowed_values.satisfied_by(&0u32));
    /// assert_eq!(true, allowed_values.satisfied_by(&1u32));
    /// assert_eq!(true, allowed_values.satisfied_by(&2u32));
    /// assert_eq!(true, allowed_values.satisfied_by(&3u32));
    /// assert_eq!(false, allowed_values.satisfied_by(&4u32));
    /// ```
    In(Vec<ConcreteT>),

    /// The concrete value must NOT be in the range of values of type ConcreteT
    /// ```
    /// # use dys_satisfiable::SatisfiableField;
    /// let disallowed_values = SatisfiableField::NotIn(vec![
    ///     1u32,
    ///     2u32,
    ///     3u32,
    /// ]);
    ///
    /// assert_eq!(true, disallowed_values.satisfied_by(&0u32));
    /// assert_eq!(false, disallowed_values.satisfied_by(&1u32));
    /// assert_eq!(false, disallowed_values.satisfied_by(&2u32));
    /// assert_eq!(false, disallowed_values.satisfied_by(&3u32));
    /// assert_eq!(true, disallowed_values.satisfied_by(&4u32));
    /// ```
    NotIn(Vec<ConcreteT>),

    /// The concrete value must be strictly greater than the value of type ConcreteT
    GreaterThan(ConcreteT),

    /// The concrete value must be greater than or equal to the value of type ConcreteT
    GreaterThanOrEqual(ConcreteT),

    /// The concrete value must be strictly less than the value of type ConcreteT
    LessThan(ConcreteT),

    /// The concrete value must be less than or equal to the value of type ConcreteT
    LessThanOrEqual(ConcreteT),

    /// The concrete value must pass a provided lambda.
    Lambda(Rc<dyn Fn(ConcreteT) -> bool>)
}

impl<ConcreteT: Clone + PartialEq + PartialOrd + Debug> Debug for SatisfiableField<ConcreteT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SatisfiableField::Lambda(_) => write!(f, "<lambda fn>"),
            SatisfiableField::Ignore => write!(f, "<ignore>"),
            SatisfiableField::Exactly(val) => write!(f, "Exactly({val:?})"),
            SatisfiableField::NotExactly(val) => write!(f, "NotExactly({val:?})"),
            SatisfiableField::In(vals) => write!(f, "In({vals:?})"),
            SatisfiableField::NotIn(vals) => write!(f, "NotIn({vals:?})"),
            SatisfiableField::GreaterThan(val) => write!(f, "GreaterThan({val:?})"),
            SatisfiableField::GreaterThanOrEqual(val) => write!(f, "GreaterThanOrEqual({val:?})"),
            SatisfiableField::LessThan(val) => write!(f, "LessThan({val:?})"),
            SatisfiableField::LessThanOrEqual(val) => write!(f, "LessThanOrEqual({val:?})"),
        }
    }
}

impl<
    ConcreteT: Clone + PartialEq + PartialOrd,
> SatisfiableField<ConcreteT> {
    pub fn satisfied_by(&self, value: &ConcreteT) -> bool {
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