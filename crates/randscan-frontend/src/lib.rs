//! RandScan Frontend - Leptos WASM Application
//!
//! A blockchain explorer frontend for RandProtocol built with Leptos.

mod app;
mod api;
mod utils;

pub use app::*;

use wasm_bindgen::prelude::*;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
