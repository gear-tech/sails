use sails_build::run_meta_dump_cli;

macro_rules! sails_services_manifest {
    ($($tt:tt)*) => {
        sails_build::service_manifest!($($tt)*)
    };
}

const SERVICE_MANIFEST: sails_build::ServiceManifest =
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/sails_services.in"));

fn main() -> anyhow::Result<()> {
    run_meta_dump_cli(SERVICE_MANIFEST.registry)
}
