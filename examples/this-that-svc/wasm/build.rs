use sails_builder::Builder;
use this_that_svc_app::MyService;

fn main() {
    Builder::new().build().generate_service_idl::<MyService>();
}
