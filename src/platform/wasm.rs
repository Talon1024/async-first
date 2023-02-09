use std::{panic, sync::Arc, future::Future};
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
    let document = window().expect("No browser window!")
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
