use crate::egl_util::{WrappedContext, WrappedDisplay};
use crate::handler::{SmithayOpenGLHandler, SmithayPlatformTaskHandler, SmithayTextInputHandler};
use crossbeam::sync::Parker;
use flutter_engine::FlutterEngine;
use log::debug;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use crate::input::keyboard::KeyboardManager;
use flutter_engine::builder::FlutterEngineBuilder;
use flutter_plugins::keyevent::KeyEventPlugin;
use flutter_plugins::textinput::TextInputPlugin;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{panic, thread};

pub trait FlutterOutputBackend {
    fn swap_buffers(&self) -> Result<(), ()>;

    fn make_current(&self) -> Result<(), ()>;

    fn get_framebuffer_dimensions(&self) -> (u32, u32);
}

pub struct FlutterOutput {
    engine: FlutterEngine,
    width: u32,
    height: u32,
}

impl Clone for FlutterOutput {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

fn create_output<B>(
    backend: B,
    options: &mut FlutterEngineOptions,
    keyboard: Arc<Mutex<KeyboardManager>>,
) -> (Parker, FlutterOutput)
where
    B: FlutterOutputBackend + Send + 'static,
{
    let (width, height) = backend.get_framebuffer_dimensions();

    let (resource_context, display) = unsafe {
        backend.make_current().expect("Invalid backend context");

        let display = WrappedDisplay::get_current();
        let resource_context = WrappedContext::create_context();
        display.clear_current();
        (resource_context, display)
    };

    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    let platform_task_handler = Arc::new(SmithayPlatformTaskHandler::new(unparker));

    let opengl_handler = SmithayOpenGLHandler::new(Box::new(backend), display, resource_context);

    let engine = FlutterEngineBuilder::new()
        .with_platform_handler(platform_task_handler)
        .with_opengl(opengl_handler)
        .with_asset_path(options.assets_path.clone())
        .with_args(options.arguments.clone())
        .build()
        .expect("Failed to create engine");

    engine.add_plugin(KeyEventPlugin::default());
    engine.add_plugin(TextInputPlugin::new(Arc::new(Mutex::new(
        SmithayTextInputHandler {
            keyboard,
            engine: engine.downgrade(),
        },
    ))));

    if let Some(callback) = options.callback.take() {
        callback(&engine);
    }

    (
        parker,
        FlutterOutput {
            engine,
            width,
            height,
        },
    )
}

fn run_output(parker: Parker, output: FlutterOutput) {
    output.engine.run().expect("Failed to start engine");

    //    let now = Instant::now();
    //    output
    //        .engine
    //        .notify_vsync(now, now + Duration::from_millis(16));

    output.engine.send_window_metrics_event(
        output.width as usize,
        output.height as usize,
        output.height as f64 / 1080.0,
    );

    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        let duration = match output.engine.execute_platform_tasks() {
            None => Duration::from_millis(100), // Just in case, wake up every so often.
            Some(tgt) => {
                let now = Instant::now();
                if tgt <= now {
                    continue;
                }
                tgt - now
            }
        };
        parker.park_timeout(duration);
    }
}

impl FlutterOutput {
    pub(crate) fn new<B>(
        backend: B,
        options: FlutterEngineOptions,
        keyboard: Arc<Mutex<KeyboardManager>>,
    ) -> Self
    where
        B: FlutterOutputBackend + Send + 'static,
    {
        debug!("Creating new flutter output");

        let (send, recv) = mpsc::channel();
        thread::spawn(move || {
            let panic_sender = send.clone();
            let mut has_sent = false;
            let result = panic::catch_unwind(AssertUnwindSafe(move || {
                let mut options = options;
                let (parker, output) = create_output(backend, &mut options, keyboard);
                send.send(Ok(output.clone())).unwrap();
                has_sent = true;
                run_output(parker, output);
            }));
            if let Err(err) = result {
                if has_sent {
                    panic::resume_unwind(err);
                } else {
                    panic_sender.send(Err(err)).unwrap();
                }
            }
        });

        match recv.recv().unwrap() {
            Ok(output) => output,
            Err(err) => panic::resume_unwind(err),
        }
    }

    pub fn engine(&self) -> FlutterEngine {
        self.engine.clone()
    }
}

pub struct FlutterEngineOptions {
    pub(crate) assets_path: PathBuf,
    pub(crate) arguments: Vec<String>,
    pub(crate) callback: Option<Box<dyn FnOnce(&FlutterEngine) + Send>>,
}

impl FlutterEngineOptions {
    pub fn new(assets_path: PathBuf, arguments: Vec<String>) -> Self {
        Self {
            assets_path,
            arguments,
            callback: None,
        }
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnOnce(&FlutterEngine) -> () + 'static + Send,
    {
        self.callback = Some(Box::new(callback));
    }
}
