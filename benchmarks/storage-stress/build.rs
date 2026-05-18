fn main() {
    const FIXED_CAPACITY: usize = 2048;

    let layout = sails_rs::build::StaticMemoryLayout::new(1024)
        .reserve_table::<32, 32>("sails_static_balances", FIXED_CAPACITY)
        .reserve_table::<64, 32>("sails_static_allowances", FIXED_CAPACITY);

    sails_rs::build::build_wasm_with_static_memory(layout);
}
