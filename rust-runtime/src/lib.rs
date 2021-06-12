#![allow(unused)]
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    unsafe {
        alert("Hello, {{project-name}}!");
    }
}

#[wasm_bindgen]
pub fn return_value() -> f64 {
    return 10.0
}

#[wasm_bindgen]
pub fn init_system() {
    console_error_panic_hook::set_once();
}

