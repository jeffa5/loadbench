pub trait InputGenerator {
    type Input: Send;
    fn next(&mut self) -> Option<Self::Input>;
    fn close(self);
}
