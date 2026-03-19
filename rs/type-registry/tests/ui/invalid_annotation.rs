use sails_type_registry::TypeInfo;

#[derive(TypeInfo)]
#[type_info(id = 42)] // Integer values are not supported in IDL V2!
struct User {
    id: u32,
}

fn main() {}
