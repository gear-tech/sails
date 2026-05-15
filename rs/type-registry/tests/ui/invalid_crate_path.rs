use sails_type_registry::TypeInfo;

#[derive(TypeInfo)]
#[type_info(crate = 123)]
struct User {
    id: u32,
}

fn main() {}
