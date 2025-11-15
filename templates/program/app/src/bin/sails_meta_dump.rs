use sails_build::{run_meta_dump_cli, service_dump};

macro_rules! sails_services {
    ($($path:path),* $(,)?) => {
        [
            $(service_dump!(
                $path,
                |entry| <$path>::__sails_entry_async(entry)
            )),*
        ]
    };
}

fn main() -> anyhow::Result<()> {
    let services = include!(concat!(env!("CARGO_MANIFEST_DIR"), "/sails_services.in"));
    run_meta_dump_cli(&services)
}
