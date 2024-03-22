
pub trait RecordType {
    fn id(&self) -> String;
}

/// Any data type that should be "snapshot" at a particular point in time.
pub trait Recordable<ToRecordT: RecordType> {
    fn to_record(&self) -> ToRecordT;
}