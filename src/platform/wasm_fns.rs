use wasm_bindgen::prelude::*;
use web_sys::EventTarget;

#[wasm_bindgen]
pub fn kill_download_button(et: &EventTarget) {
	web_sys::console::log_1(et);
}
