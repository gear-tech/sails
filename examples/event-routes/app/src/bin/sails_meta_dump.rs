use sails_build::run_meta_dump_cli;

const SERVICE_MANIFEST: sails_build::ServiceManifest = include!(concat!(env!("OUT_DIR"), "/", sails_build::generated_manifest_file!()));

fn main() -> anyhow::Result<()> {
    run_meta_dump_cli(SERVICE_MANIFEST.registry)
}
