pub trait JsonBuilder {
    fn build(self) -> Vec<u8>;
    fn build_and_get_slice(&mut self) -> &[u8];
}
