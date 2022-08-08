pub trait JsonBuilder {
    fn build(self) -> Vec<u8>;
    fn build_and_clone(&self) -> Vec<u8>;
}
