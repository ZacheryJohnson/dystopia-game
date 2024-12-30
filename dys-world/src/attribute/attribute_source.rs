use crate::attribute::attribute_type::AttributeType;
use crate::attribute::instance::{AttributeInstance, AttributeValueT};

/// Game data that can modify the attributes of a combatant.
pub trait AttributeSource {
    /// Name of the source providing the stats.
    /// For example, limbs would provide the name of the limb ("Arm").
    fn source_name(&self) -> String;

    /// Returns the value of a particular attribute type provided by the source, if it exists.
    fn attribute_total(&self, attribute_type: &AttributeType) -> Option<AttributeValueT>;

    /// Gets all attributes provided by the source.
    fn attributes(&self) -> Vec<AttributeInstance>;
}