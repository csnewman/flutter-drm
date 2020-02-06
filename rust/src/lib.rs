mod egl_util;
pub(crate) mod handler;
pub(crate) mod input;
pub mod output;
pub mod udev;
pub mod winit;

use flutter_engine::{FlutterEngine, FlutterEngineWeakRef};
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use smithay::reexports::calloop::EventLoop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct FlutterDrmManager {
    pub(crate) event_loop: EventLoop<()>,
}

impl FlutterDrmManager {
    pub fn new() -> Self {
        Self {
            event_loop: EventLoop::<()>::new().unwrap(),
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

pub struct EngineWeakCollection {
    engines: Arc<RwLock<Vec<FlutterEngineWeakRef>>>,
}

impl Clone for EngineWeakCollection {
    fn clone(&self) -> Self {
        Self {
            engines: self.engines.clone(),
        }
    }
}

impl EngineWeakCollection {
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, engine: FlutterEngineWeakRef) {
        self.engines.write().push(engine);
    }

    pub fn for_each<F>(&self, func: F)
    where
        F: Fn(FlutterEngine),
    {
        let engines = self.engines.upgradable_read();
        let mut dirty = false;

        for engine in engines.iter() {
            match engine.upgrade() {
                None => dirty = true,
                Some(engine) => func(engine),
            }
        }

        // We detected some engines failed to upgrade, clear bad references
        if dirty {
            let mut engines = RwLockUpgradableReadGuard::upgrade(engines);
            engines.retain(|e| e.is_valid());
        }
    }
}
