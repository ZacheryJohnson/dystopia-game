use std::fmt::Display;
use serde::Serialize;

pub trait RecordType: Serialize {
    const RECORD_PREFIX: &'static str;
    type InstanceIdType: Display;

    /// Each record type has a shared identifier between records.
    /// For example, with two types Car and Pet, the type_id of Car may be "CAR", while the type_id
    /// of Pet may be "PET". With different instances of Cars, both would have a type_id of "CAR",
    /// but different instance_ids.
    fn type_id(&self) -> String {
        format!("{}-{}", Self::RECORD_PREFIX, self.instance_id())
    }

    /// Instance ID the record was derived from.
    fn instance_id(&self) -> Self::InstanceIdType;
}

/// Any data type that should be "snapshot" at a particular point in time.
pub trait Recordable<RecordT: RecordType> {
    fn to_record(&self) -> RecordT;
}
