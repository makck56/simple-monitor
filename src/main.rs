/*!
    A very simple application that show your name in a message box.
    See `basic_d` for the derive version
*/
#![windows_subsystem = "windows"]
extern crate native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::Cell;
use std::time::Duration;
use winapi::shared::windef::POINT;
use winapi::um::winuser::GetCursorPos;
use winapi::um::winuser::*;

mod sys;

#[derive(Default)]
pub struct BasicApp {
    window: nwg::Window,
    mem_label: nwg::Label,
    tray_item: nwg::TrayNotification,
    timer: nwg::AnimationTimer,
    is_dragging: Cell<bool>,
    drag_start: Cell<(i32, i32)>,
}

impl BasicApp {
    fn say_goodbye(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn get_cursor_position() -> Option<(i32, i32)> {
    unsafe {
        let mut point: POINT = std::mem::zeroed(); // 初始化 POINT 结构体
        if GetCursorPos(&mut point) == 0 {
            None // 获取失败
        } else {
            Some((point.x, point.y)) // 返回鼠标的 x, y 坐标
        }
    }
}

mod basic_app_ui {
    use super::*;
    use native_windows_gui as nwg;
    use nwg::MousePressEvent;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct BasicAppUi {
        inner: Rc<BasicApp>,
        default_handler: RefCell<Option<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<BasicAppUi> for BasicApp {
        fn build_ui(mut data: BasicApp) -> Result<BasicAppUi, nwg::NwgError> {
            use nwg::Event as E;
            // Controls
            nwg::Window::builder()
                .flags(nwg::WindowFlags::POPUP | nwg::WindowFlags::VISIBLE)
                .ex_flags(WS_EX_LAYERED)
                .size((300, 135))
                .position((300, 300))
                .title("Basic example")
                .build(&mut data.window)?;

            nwg::Label::builder()
                .size((280, 30)) // 设置大小
                .position((10, 20)) // 设置位置
                .text("0%") // 设置文本
                .parent(&data.window) // 设置父控件
                .build(&mut data.mem_label)
                .unwrap();

            let icon = nwg::Icon::from_file("E:\\Code\\basic\\src\\icon.ico", false).unwrap();

            nwg::TrayNotification::builder()
                .parent(&data.window)
                .icon(Some(&icon))
                .tip(Some("Hello"))
                .build(&mut data.tray_item)
                .expect("Failed to build UI");

            nwg::AnimationTimer::builder()
                .parent(&data.window)
                .interval(Duration::from_millis(1000))
                .build(&mut data.timer)
                .expect("Failed to Build Timer");

            // Wrap-up
            let ui = BasicAppUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };
            // 获取窗口句柄
            let hwnd = ui.window.handle.hwnd().unwrap();
            // 设置透明度，15% 的透明度相当于 255 * 0.15 ≈ 38
            // 设定特定颜色透明
            let alpha = (255.0 * 0.50) as u8; // 设置 85% 的透明度

            unsafe {
                SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
            }

            // Events
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose => {
                            if &handle == &ui.window {
                                BasicApp::say_goodbye(&ui);
                            }
                        }
                        E::OnMousePress(e) => match e {
                            MousePressEvent::MousePressLeftDown => {
                                let point = get_cursor_position().unwrap();
                                ui.drag_start.set(point);
                                ui.is_dragging.set(true);
                            }
                            MousePressEvent::MousePressLeftUp => {
                                ui.is_dragging.set(false);
                            }
                            _ => {}
                        },
                        E::OnMouseMove => {
                            if ui.is_dragging.get() == true {
                                let point = get_cursor_position().unwrap();
                                let (x, y) = point;
                                ui.window.set_position(x - 100, y - 100);
                            }
                        }
                        E::OnTimerTick => {
                            if &handle == &ui.timer {
                                let mem_status = sys::sys_info::get_memory_status();

                                let total_memory =
                                    mem_status.mem_total as f64 / (1024 * 1024 * 1024) as f64; // 总物理内存
                                let used_memory =
                                    mem_status.mem_used as f64 / (1024 * 1024 * 1024) as f64; // 可用物理内存
                                let memory_load = mem_status.memory_load; // 内存使用百分比
                                ui.mem_label.set_text(&format!(
                                    "Memory: {}% ({:.2}G / {:.2}G)",
                                    memory_load, used_memory, total_memory
                                ));
                                return ();
                            }
                        }
                        _ => {}
                    }
                }
            };

            *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
                &ui.window.handle,
                handle_events,
            ));
            ui.timer.start();

            return Ok(ui);
        }
    }

    impl Drop for BasicAppUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let handler = self.default_handler.borrow();
            if handler.is_some() {
                nwg::unbind_event_handler(handler.as_ref().unwrap());
            }
        }
    }

    impl Deref for BasicAppUi {
        type Target = BasicApp;

        fn deref(&self) -> &BasicApp {
            &self.inner
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _ui = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
