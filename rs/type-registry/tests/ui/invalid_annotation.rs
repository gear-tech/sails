use sails_type_registry::TypeInfo;

#[derive(TypeInfo)]
#[type_info(id = 42)] // Integer values are not supported
struct User {
    id: u32,
}

fn main() {}
