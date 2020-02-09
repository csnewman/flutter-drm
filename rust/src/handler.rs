use crate::egl_util::{WrappedContext, WrappedDisplay};
use async_std::task;
use flutter_engine::{FlutterEngineHandler, FlutterEngineWeakRef};
use futures_task::FutureObj;
use std::future::Future;
use std::os::raw::c_void;
use std::sync::{Arc, Weak};

use crate::input::keyboard::KeyboardManager;
use crate::output::FlutterOutputBackend;
use crossbeam::sync::Unparker;
use flutter_plugins::textinput::TextInputHandler;
use parking_lot::Mutex;
use smithay::backend::egl::ffi;

pub struct SmithayFlutterHandler {
    pub backend: Weak<dyn FlutterOutputBackend>,
    pub display: WrappedDisplay,
    pub resource_context: WrappedContext,
    pub unparker: Unparker,
}

unsafe impl Send for SmithayFlutterHandler {}

unsafe impl Sync for SmithayFlutterHandler {}

impl FlutterEngineHandler for SmithayFlutterHandler {
    fn swap_buffers(&self) -> bool {
        match self.backend.upgrade() {
            None => false,
            Some(backend) => match backend.swap_buffers() {
                Ok(_) => true,
                Err(_) => false,
            },
        }
    }

    fn make_current(&self) -> bool {
        match self.backend.upgrade() {
            None => false,
            Some(backend) => match backend.make_current() {
                Ok(_) => true,
                Err(_) => false,
            },
        }
    }

    fn clear_current(&self) -> bool {
        unsafe {
            // TODO: Expose result
            self.display.clear_current();
            true
        }
    }

    fn fbo_callback(&self) -> u32 {
        0
    }

    fn make_resource_current(&self) -> bool {
        unsafe { self.resource_context.make_current() }
    }

    fn gl_proc_resolver(&self, proc: *const i8) -> *mut c_void {
        unsafe { ffi::egl::GetProcAddress(proc) as _ }
    }

    fn wake_platform_thread(&self) {
        self.unparker.unpark();
    }

    fn run_in_background(&self, func: Box<dyn Future<Output = ()> + Send + 'static>) {
        task::spawn(FutureObj::new(func));
    }
}

pub struct SmithayTextInputHandler {
    pub keyboard: Arc<Mutex<KeyboardManager>>,
    pub engine: FlutterEngineWeakRef,
}

impl TextInputHandler for SmithayTextInputHandler {
    fn show(&mut self) {
        self.keyboard.lock().set_text_target(self.engine.clone());
    }

    fn hide(&mut self) {
        self.keyboard.lock().clear_text_target(self.engine.clone());
    }
}
