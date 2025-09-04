pub trait JsonValueWriter {
    const IS_ARRAY: bool;
    fn write(&self, dest: &mut String);
}
