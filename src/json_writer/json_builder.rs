pub trait JsonBuilder {
    fn build(self) -> Vec<u8>;
    fn build_into(&self, dest: &mut Vec<u8>);
}
