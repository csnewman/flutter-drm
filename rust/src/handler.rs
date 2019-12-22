use crate::egl_util::{WrappedContext, WrappedDisplay};
use async_std::task;
use flutter_engine::ffi::ExternalTextureFrame;
use flutter_engine::FlutterEngineHandler;
use futures_task::FutureObj;
use std::future::Future;
use std::os::raw::c_void;
use std::sync::Weak;

use crate::output::FlutterOutputBackend;
use crossbeam::sync::Unparker;
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

    fn get_texture_frame(
        &self,
        texture_id: i64,
        size: (usize, usize),
    ) -> Option<ExternalTextureFrame> {
        unimplemented!()
    }
}
