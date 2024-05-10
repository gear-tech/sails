use rmrk_catalog_app::Program;
use sails_builder::Builder;

fn main() {
    Builder::new().build().generate_program_idl::<Program>();
}
