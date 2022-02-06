pub trait JsonBuilder {
    fn build(self) -> Vec<u8>;
}
