pub trait JsonValue {
    const IS_ARRAY: bool;
    fn write(&self, dest: &mut String);
}
