#![recursion_limit = "1024"]
mod api;
mod app;
mod components;
mod types;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[macro_use]
extern crate dotenv_codegen;

extern crate inflector;

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<app::App>::new().mount_to_body();
    wasm_logger::init(wasm_logger::Config::default());
}
