//! Test that #[reflect_hash(crate = ...)] attribute works

// Re-export sails_reflect_hash under a different name
use sails_reflect_hash as my_custom_hash;
use my_custom_hash::ReflectHash;

#[derive(ReflectHash)]
#[reflect_hash(crate = my_custom_hash)]
struct CustomPathStruct {
    value: u32,
}

#[derive(ReflectHash)]
#[reflect_hash(crate = my_custom_hash)]
enum CustomPathEnum {
    A,
    B(u32),
    C { field: u64 },
}

#[test]
fn test_custom_crate_path() {
    // Just verify that it compiles and we can access the HASH constant
    let _ = CustomPathStruct::HASH;
    let _ = CustomPathEnum::HASH;
    
    // Verify they produce different hashes
    assert_ne!(CustomPathStruct::HASH, CustomPathEnum::HASH);
}
