use sails_type_registry::TypeInfo;

#[derive(TypeInfo)]
union MyUnion {
    a: u32,
    b: f32,
}

fn main() {}
