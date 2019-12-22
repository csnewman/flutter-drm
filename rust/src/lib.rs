mod egl_util;
pub(crate) mod handler;
pub mod output;
pub mod udev;
pub mod winit;

use smithay::reexports::calloop::{
    generic::{EventedFd, Generic},
    EventLoop, LoopHandle, Source,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct FlutterDrmManager {
    pub(crate) event_loop: EventLoop<()>,
}

impl FlutterDrmManager {
    pub fn new() -> Self {
        Self {
            event_loop: EventLoop::<()>::new().unwrap()
        }
    }

    pub fn run(&mut self) {
        let running = Arc::new(AtomicBool::new(true));

        while running.load(Ordering::SeqCst) {
            if self.event_loop.dispatch(None, &mut ()).is_err() {
                running.store(false, Ordering::SeqCst);
            }
        }
    }
}
