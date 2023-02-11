use std::sync::Arc;
use glow::HasContext;
use platform::DrawingContextRequest;
use rfd::AsyncFileDialog;
use winit::{
    window::WindowBuilder,
    event_loop::EventLoopBuilder,
    event::WindowEvent::*,
    event::Event,
};
use egui_glow::EguiGlow;
#[cfg(target_family = "wasm")]
use wasm_bindgen::JsValue;

/* 
async fn some_random_num() -> u8 {
    let mut the_byte = 7;
    getrandom::getrandom(slice::from_mut(&mut the_byte)).unwrap();
    the_byte
}
 */

#[derive(Debug, Clone)]
enum AppEvent {
    NewMessage(String),
}

pub fn main() {
    platform::init();
    let el = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let elp = el.create_proxy();
    let win = WindowBuilder::new().with_title("Practice").build(&el)
        .expect("Unable to create window!");
    let (wc, glc) = platform::show_window(&win, &el, DrawingContextRequest::OpenGL);
    let mut show_quit_window = false;
    let mut message = Some(String::from("The first line of the selected file will be appended to this string."));
    let mut egui_glow = EguiGlow::new(&el, Arc::clone(&glc), None);
    el.run(move |event, _el, control_flow| {
        match event {
            Event::WindowEvent { window_id: _, event } => {
                if egui_glow.on_event(&event).consumed { return (); }
                match event {
                    Resized(new_size) => {
                        unsafe { glc.viewport(0, 0, new_size.width as i32, new_size.height as i32) };
                    },
                    CloseRequested => {
                        control_flow.set_exit_with_code(0);
                    },
                    DroppedFile(fpath) => {
                        let fpath = fpath.to_str().unwrap_or("<Invalid UTF-8>");
                        #[cfg(target_family = "wasm")]
                        {
                            let jsvalue = JsValue::from_str(fpath);
                            web_sys::console::log_1(&jsvalue);
                        }
                        #[cfg(not(target_family = "wasm"))]
                        {
                            println!("{fpath}");
                        }
                    },
                    _ => (),
                }
            },
            Event::UserEvent(ue) => {
                match ue {
                    AppEvent::NewMessage(new_msg) => {
                        let msg = &mut message;
                        let msg = msg.get_or_insert_with(Default::default);
                        if !msg.is_empty() {
                            msg.push('\n');
                        }
                        msg.push_str(&new_msg);
                    },
                }
            }
            Event::MainEventsCleared => {
                unsafe {
                    glc.clear_color(0.0, 0.0, 0.0, 1.0);
                    glc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }
                egui_glow.run(&win, |ctx| {
                    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            egui::menu::menu_button(ui, "File", |ui| {
                                if ui.button("Open").clicked() {
                                    let elp = elp.clone(); // Prevent Send/Sync-related issues
                                    platform::spawn_local(async move {
                                        let file_handle = AsyncFileDialog::new().pick_file().await;
                                        let return_message = if let Some(file_handle) = file_handle {
                                            let file_contents = file_handle.read().await;
                                            let file_text = String::from_utf8(file_contents);
                                            if let Ok(text) = file_text {
                                                text.lines().next().unwrap().to_string()
                                            } else {
                                                let fname = file_handle.file_name();
                                                format!("File {fname} is not a text file")
                                            }
                                        } else {
                                            String::from("No file chosen.")
                                        };
                                        elp.send_event(AppEvent::NewMessage(return_message)).unwrap();
                                    });
                                    ui.close_menu();
                                }
                                if ui.button("Save").clicked() {
                                    let contents = message.clone().unwrap_or(String::new()).into_bytes();
                                    platform::spawn_local(platform::save_file(contents, ""));
                                    ui.close_menu();
                                }
                                if ui.button("Exit").clicked() {
                                    #[cfg(not(target_family = "wasm"))]
                                    { control_flow.set_exit_with_code(0); }
                                    #[cfg(target_family = "wasm")]
                                    { show_quit_window = true; }
                                    ui.close_menu();
                                }
                            });
                        })
                    });
                    let msg_window = egui::Window::new("Message").default_height(400.0);
                    {
                        let msg = &mut message;
                        if msg.is_some() {
                            msg_window.show(ctx, |ui| {
                                if ui.button("Clear").clicked() {
                                    *msg = None;
                                } else {
                                    ui.label(msg.as_ref().unwrap());
                                }
                            });
                        }
                    }
                    let quit_window = egui::Window::new("Quit").default_height(400.0).collapsible(false);
                    if show_quit_window {
                        quit_window.show(ctx, |ui| {
                            ui.label("Close this browser tab to quit the app.");
                        });
                    }
                });
                egui_glow.paint(&win);
                // SWAP BUFFERS
                // ==================================================================
                if let Err(e) = wc.swap_buffers() {
                    eprintln!("{:?}", e);
                }
            },
            // Event::RedrawRequested(_) => todo!(),
            // Event::RedrawEventsCleared => todo!(),
            Event::LoopDestroyed => (),
            _ => (),
        }
    });
}

mod platform;
