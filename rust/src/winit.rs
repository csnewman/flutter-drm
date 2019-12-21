use crate::egl_util::WrappedDisplay;
use ::winit::{dpi::LogicalSize, WindowBuilder};
use log::debug;
use smithay::backend::graphics::gl::GLGraphicsBackend;
use smithay::backend::winit;
use std::sync::Arc;

use crate::output::{FlutterOutput, FlutterOutputBackend};
use smithay::backend::input::InputBackend;
use smithay::backend::winit::WinitGraphicsBackend;
use smithay::reexports::calloop::EventLoop;
use std::sync::atomic::{AtomicBool, Ordering};

struct WinitOutputBackend {
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

pub fn new_winit() {
    let mut event_loop = EventLoop::<()>::new().unwrap();
    let signal = event_loop.get_signal();

    // ------------

    debug!("Creating window");
    let (graphics, mut input) = winit::init_from_builder(
        WindowBuilder::new()
            .with_dimensions(LogicalSize::new(1280.0 / 1.5, 800.0 / 1.5))
            .with_resizable(false)
            .with_title("Flutter Compositor")
            .with_visibility(true),
        None,
    )
    .unwrap();

    // Winit leaves the window bound
    unsafe {
        let display = WrappedDisplay::get_current();
        display.clear_current();
    }

    let backend = Arc::new(WinitOutputBackend { graphics });
    let output = FlutterOutput::new(backend.clone() as _);

    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        input.dispatch_new_events().unwrap();

        if event_loop
            .dispatch(Some(::std::time::Duration::from_millis(16)), &mut ())
            .is_err()
        {
            running.store(false, Ordering::SeqCst);
        }
    }
}
