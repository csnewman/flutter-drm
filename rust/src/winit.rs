use crate::egl_util::WrappedDisplay;
use log::debug;
use smithay::backend::graphics::gl::GLGraphicsBackend;
use smithay::backend::winit;
use std::sync::Arc;

use crate::output::{FlutterEngineOptions, FlutterOutput, FlutterOutputBackend};
use crate::FlutterDrmManager;
use smithay::backend::input::InputBackend;
use smithay::backend::winit::{
    WinitEventsHandler, WinitGraphicsBackend, WinitInputBackend, WinitInputError,
};
use smithay::reexports::calloop::EventLoop;
use std::sync::atomic::{AtomicBool, Ordering};

pub use ::winit::{dpi::LogicalSize, dpi::PhysicalSize, WindowBuilder};

use flutter_engine::FlutterEngine;
use log::info;
use std::thread;
use std::time::Duration;

pub struct WinitOutputBackend {
    graphics: WinitGraphicsBackend,
}

impl FlutterOutputBackend for WinitOutputBackend {
    fn swap_buffers(&self) -> Result<(), ()> {
        self.graphics.swap_buffers().map_err(|_| ())
    }

    fn make_current(&self) -> Result<(), ()> {
        unsafe { self.graphics.make_current().map_err(|_| ()) }
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.graphics.get_framebuffer_dimensions()
    }
}

unsafe impl Sync for WinitOutputBackend {}
unsafe impl Send for WinitOutputBackend {}

struct InputWrapper(WinitInputBackend);
unsafe impl Send for InputWrapper {}

pub struct WinitOutputManager {}

impl WinitOutputManager {
    pub fn new(manager: &FlutterDrmManager) -> Self {
        Self {}
    }

    pub fn create_window(
        &self,
        builder: WindowBuilder,
        options: FlutterEngineOptions,
    ) -> FlutterOutput<WinitOutputBackend> {
        debug!("Creating window");
        let (graphics, mut input) = winit::init_from_builder(builder, None).unwrap();

        // Winit leaves the window bound
        unsafe {
            let display = WrappedDisplay::get_current();
            display.clear_current();
        }

        let backend = Arc::new(WinitOutputBackend { graphics });
        let output = FlutterOutput::new(backend, options);

        input.set_events_handler(WinitOutputEventsHandler {
            engine: output.engine(),
        });

        let input = InputWrapper(input);
        thread::spawn(move || {
            let mut input = input.0;
            let mut running = true;

            while running {
                match input.dispatch_new_events() {
                    Ok(_) => {}
                    Err(WinitInputError::WindowClosed) => {
                        running = false;
                        // TODO: Signal
                    }
                }
                thread::sleep(Duration::from_millis(16));
            }
        });

        output
    }
}

struct WinitOutputEventsHandler {
    engine: FlutterEngine,
}

impl WinitEventsHandler for WinitOutputEventsHandler {
    fn resized(&mut self, size: (f64, f64), scale: f64) {
        debug!("Window resized: {:?}x{}", size, scale);
        self.engine
            .send_window_metrics_event(size.0 as i32, size.1 as i32, 1.0);
    }

    fn focus_changed(&mut self, focused: bool) {}

    fn refresh(&mut self) {}
}
