pub trait Service {
    type Exposure;

    fn expose(self, route: &'static [u8]) -> Self::Exposure;
}
