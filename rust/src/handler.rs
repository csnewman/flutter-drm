use crate::egl_util::{WrappedContext, WrappedDisplay};
use flutter_engine::{FlutterEngineWeakRef, FlutterOpenGLHandler};
use std::os::raw::c_void;
use std::sync::Arc;

use crate::input::keyboard::KeyboardManager;
use crate::output::FlutterOutputBackend;
use crossbeam::sync::Unparker;
use flutter_engine::tasks::TaskRunnerHandler;
use flutter_plugins::textinput::TextInputHandler;
use parking_lot::Mutex;
use smithay::backend::egl::ffi;

pub(crate) struct SmithayPlatformTaskHandler {
    unparker: Unparker,
}

impl SmithayPlatformTaskHandler {
    pub(crate) fn new(unparker: Unparker) -> Self {
        Self { unparker }
    }
}

impl TaskRunnerHandler for SmithayPlatformTaskHandler {
    fn wake(&self) {
        self.unparker.unpark();
    }
}

pub(crate) struct SmithayOpenGLHandler {
    backend: Box<dyn FlutterOutputBackend + Send>,
    display: WrappedDisplay,
    resource_context: WrappedContext,
}

impl SmithayOpenGLHandler {
    pub(crate) fn new(
        backend: Box<dyn FlutterOutputBackend + Send>,
        display: WrappedDisplay,
        resource_context: WrappedContext,
    ) -> Self {
        Self {
            backend,
            display,
            resource_context,
        }
    }
}

impl FlutterOpenGLHandler for SmithayOpenGLHandler {
    fn swap_buffers(&self) -> bool {
        match self.backend.swap_buffers() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn make_current(&self) -> bool {
        match self.backend.make_current() {
            Ok(_) => true,
            Err(_) => false,
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
