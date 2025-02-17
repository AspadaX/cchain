pub trait HumanReadable {
    /// Get access to the raw name of the subject
    fn get_raw_name(&self) -> String;

    /// Get a human readable form of the raw name
    fn get_human_readable_name(&self) -> String;
}
