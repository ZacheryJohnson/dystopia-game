pub trait AttributeDisplay {
    /// Should this attribute be displayed?
    /// This can vary depending on the attribute, as there may be stats hidden from players
    /// or we may wish to hide attribute that are zero
    fn should_display(&self) -> bool;
}