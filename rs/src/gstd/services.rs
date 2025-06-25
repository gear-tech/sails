pub trait Service {
    type Exposure: Exposure;

    fn expose(self, route: &'static [u8]) -> Self::Exposure;
}

pub trait Exposure {
    fn route(&self) -> &'static [u8];
    fn check_asyncness(input: &[u8]) -> Option<bool>;
}
