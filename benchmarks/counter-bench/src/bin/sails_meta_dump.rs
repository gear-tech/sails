use sails_build::{run_meta_dump_cli, service_dump};

macro_rules! sails_services {
    (
        $(type $alias:ident = $ty:ty;)*
        services: [
            $($path:path),* $(,)?
        ] $(,)?
    ) => {
        $(#[allow(dead_code)] pub type $alias = $ty;)*
        pub const SAILS_SERVICE_REGISTRY: &[sails_build::DumpService] = &[
            $(service_dump!(
                $path,
                |entry| <$path>::__sails_entry_async(entry)
            )),*
        ];
    };
    ($($path:path),* $(,)?) => {
        sails_services! {
            services: [ $($path),* ]
        }
    };
}

mod sails_services_manifest {
    use super::*;
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/sails_services.in"));
}

fn main() -> anyhow::Result<()> {
    run_meta_dump_cli(sails_services_manifest::SAILS_SERVICE_REGISTRY)
}
