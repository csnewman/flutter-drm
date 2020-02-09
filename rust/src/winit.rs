use crate::egl_util::WrappedDisplay;
use log::debug;
use smithay::backend::graphics::gl::GLGraphicsBackend;
use smithay::backend::winit;
use std::sync::Arc;

use crate::output::{FlutterEngineOptions, FlutterOutput, FlutterOutputBackend};
use crate::{EngineWeakCollection, FlutterDrmManager};
use smithay::backend::input::InputBackend;
use smithay::backend::winit::{
    WinitEventsHandler, WinitGraphicsBackend, WinitInputBackend, WinitInputError,
};

pub use ::winit::{dpi::LogicalSize, dpi::PhysicalSize, WindowBuilder};

use crate::input::keyboard::KeyboardManager;
use crate::input::winit::WinitInputHandler;
use flutter_engine::FlutterEngine;
use parking_lot::Mutex;
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

pub struct WinitOutputManager {
    engines: EngineWeakCollection,
    keyboard: Arc<Mutex<KeyboardManager>>,
}

impl WinitOutputManager {
    pub fn new(manager: &FlutterDrmManager) -> Self {
        let engines = EngineWeakCollection::new();

        Self {
            keyboard: Arc::new(Mutex::new(KeyboardManager::new(engines.clone()))),
            engines,
        }
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

        // Create output
        let backend = Arc::new(WinitOutputBackend { graphics });
        let output = FlutterOutput::new(backend, options, self.keyboard.clone());
        let engine = output.engine();
        self.engines.add(engine.downgrade());

        // Configure input
        input.set_events_handler(WinitOutputEventsHandler { engine });
        input.set_handler(WinitInputHandler::new(self.keyboard.clone()));

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
        self.engine.run_on_platform_thread(move |engine| {
            engine.send_window_metrics_event(size.0 as usize, size.1 as usize, 1.0);
        });
    }

    fn focus_changed(&mut self, focused: bool) {}

    fn refresh(&mut self) {}
}
