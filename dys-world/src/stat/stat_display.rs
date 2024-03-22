pub trait StatDisplay {
    /// Should this stat be displayed?
    /// This can vary depending on the stat, as there may be stats hidden from players
    /// or we may wish to hide stats that are zero
    fn should_display(&self) -> bool;
}