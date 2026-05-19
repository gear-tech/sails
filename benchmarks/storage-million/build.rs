fn main() {
    const MILLION_BALANCE_LOG2_SLOTS: u8 = 21;
    const MILLION_BALANCE_SLOTS: usize = 1usize << MILLION_BALANCE_LOG2_SLOTS;
    const PAGE_LOCAL_BALANCE_LOG2_TILES: u8 = 13;

    let layout = sails_rs::build::StaticMemoryLayout::new(1024)
        .reserve_table::<32, 32>("million_balances", MILLION_BALANCE_SLOTS)
        .reserve_table::<64, 32>("million_allowances", MILLION_BALANCE_SLOTS)
        .reserve_actor_u256_map::<MILLION_BALANCE_LOG2_SLOTS>("wat_actor_balances")
        .reserve_actor_u256_map::<MILLION_BALANCE_LOG2_SLOTS>("mixed_actor_balances")
        .reserve_allowance_u256_map::<MILLION_BALANCE_LOG2_SLOTS>("wat_allowances")
        .reserve_control_actor_u256_map::<MILLION_BALANCE_LOG2_SLOTS>("control_actor_balances")
        .reserve_page_local_actor_u256_map::<PAGE_LOCAL_BALANCE_LOG2_TILES>(
            "page_local_actor_balances",
        )
        .reserve_grouped_control_actor_u256_map::<12, 1>("grouped_actor_balances_pages2")
        .reserve_grouped_control_actor_u256_map::<11, 2>("grouped_actor_balances_pages4")
        .reserve_grouped_control_actor_u256_map::<10, 3>("grouped_actor_balances_pages8")
        .reserve_grouped_control_actor_u256_map::<9, 4>("grouped_actor_balances_pages16")
        .reserve_grouped_control_actor_u256_map::<8, 5>("grouped_actor_balances_pages32")
        .reserve_grouped_control_actor_u256_map::<7, 6>("grouped_actor_balances_pages64")
        .reserve_grouped_control_actor_u256_map::<6, 7>("grouped_actor_balances_pages128");

    sails_rs::build::build_wasm_with_static_memory(layout);
}
