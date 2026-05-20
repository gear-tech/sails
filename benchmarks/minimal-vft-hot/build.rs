fn main() {
    const VFT_LOG2_SLOTS: u8 = 21;

    let layout = sails_rs::build::StaticMemoryLayout::new(1024)
        .reserve_actor_u256_map::<VFT_LOG2_SLOTS>("balances")
        .reserve_actor_pair_u256_map::<VFT_LOG2_SLOTS>("allowances");

    sails_rs::build::build_wasm_with_static_memory(layout);
}
