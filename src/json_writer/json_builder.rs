pub trait JsonObject {
    fn write_into(&self, dest: &mut Vec<u8>);
}
