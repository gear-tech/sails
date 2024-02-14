#![no_std]

use rmrk_catalog_app::{requests, Catalog, CatalogData};
use sails_rtl_gstd::{gstd, gstd::msg, GStdExecContext};

static mut CATALOG_DATA: Option<CatalogData> = None;

#[no_mangle]
extern "C" fn init() {
    let catalog_data = unsafe {
        CATALOG_DATA = Some(CatalogData::default());
        CATALOG_DATA.as_mut().unwrap()
    };
    let exec_context = GStdExecContext::new();
    Catalog::new(catalog_data, exec_context);
}

#[gstd::async_main] // Make async optional
async fn main() {
    let catalog_data = unsafe { CATALOG_DATA.as_mut().unwrap() };
    let exec_context = GStdExecContext::new();
    let input_bytes = msg::load_bytes().expect("Failed to read input");
    let output_bytes =
        requests::process(&mut Catalog::new(catalog_data, exec_context), &input_bytes).await;
    msg::reply_bytes(output_bytes, 0).expect("Failed to send output");
}
