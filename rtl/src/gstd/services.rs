pub trait Service {
    type Exposed;

    fn expose(self, invocation_route: &'static [u8]) -> Self::Exposed;
}
