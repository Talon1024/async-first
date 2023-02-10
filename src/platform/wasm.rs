use std::{panic, sync::Arc, future::Future};
use base64::Engine;
use glow::Context;
use wasm_bindgen::{prelude::*, JsValue};
use web_sys::{window, WebGl2RenderingContext, console, WebGlRenderingContext};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::Window,
};
use winit::platform::web::WindowExtWebSys;
use super::DrawingContextRequest;

pub(crate) struct WindowContext;

impl WindowContext {
    pub(crate) fn swap_buffers(&self) -> Result<(), ()> {
        Ok(())
    }
}

pub(crate) fn spawn_local(f: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(f)
}

pub(crate) fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

pub(crate) fn show_window<CE>(win: &Window, _el: &EventLoopWindowTarget<CE>, _ctxr: DrawingContextRequest) -> (WindowContext, Arc<Context>) {
    let canvas = win.canvas();
    let document = window().expect("No window!")
        .document().expect("No document!");
    document.body().expect("Failed to get document body!")
        .append_child(&canvas)
        .expect("Failed to add canvas to document body!");
    let glc = canvas.get_context("webgl2").and_then(|ctx| {
        ctx.ok_or(JsValue::from_str("WebGL2 context not created"))
            .and_then(|ctx| {
                ctx.dyn_into::<WebGl2RenderingContext>()
                .map_err(JsValue::from)
            })
            .map(Context::from_webgl2_context)
            .map_err(|e| JsValue::from_str(&format!("Failed to get WebGL2 context!\n\n{e:?}")))
    }).or_else(|e| {
        console::error_1(&e);
        canvas.get_context("webgl").and_then(|ctx| {
            ctx.ok_or(JsValue::from_str("WebGL context not created"))
                .and_then(|ctx| {
                    ctx.dyn_into::<WebGlRenderingContext>()
                    .map_err(JsValue::from)
                })
                .map(Context::from_webgl1_context)
                .map_err(|e| JsValue::from_str(&format!("Failed to get WebGL context!\n\n{e:?}")))
        })
    }).map(Arc::new).unwrap();
    (WindowContext, glc)
}

/* 
macro_rules! set_attribute {
    ($element: ident, $attribute: literal, $value: expr) => {
        let 
        $element.set_attribute($attribute, $value).expect("{}")
    };
}
 */

pub(crate) async fn save_file(contents: Vec<u8>, fname: &str, ftype: &str) {
    let bwindow = window().expect("No window!");
    let document = bwindow.document().expect("No document!");
    let body = document.body().expect("No body!");
    let anchor = document.create_element("a").expect("`a` should be a valid element name");
    let a_text = document.create_text_node("💾 Save");
    anchor.append_child(&a_text).expect("No text on download button");
    // Download and class attributes
    anchor.set_attribute("download", fname).expect("`download` should be a valid attribute name");
    anchor.set_attribute("class", "download_button").expect("`class` should be a valid attribute name");
    // The data to download
    let base64 = if ftype == "text/plain" { "" } else { ";base64" };
    let data = if !base64.is_empty() {
        let engine = base64::engine::general_purpose::URL_SAFE;
        engine.encode(contents)
    } else {
        percent_encoding::percent_encode(&contents, percent_encoding::NON_ALPHANUMERIC).collect()
    };
    let href = format!("data:{ftype}{base64},{data}");
    anchor.set_attribute("href", &href).expect("`href` should be a valid attribute name");
    // Event listeners
    // anchor.add_event_listener_with_callback("mouseup", &super::wasm_fns::kill_download_button);
    body.append_child(&anchor).expect("Could not add anchor to document");
}
