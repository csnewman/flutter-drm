use crate::egl_util::{WrappedContext, WrappedDisplay};
use crate::handler::{SmithayFlutterHandler, SmithayTextInputHandler};
use crossbeam::sync::Parker;
use flutter_engine::FlutterEngine;
use log::debug;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use crate::input::keyboard::KeyboardManager;
use flutter_plugins::keyevent::KeyEventPlugin;
use flutter_plugins::textinput::TextInputPlugin;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::{panic, thread};

pub trait FlutterOutputBackend {
    fn swap_buffers(&self) -> Result<(), ()>;

    fn make_current(&self) -> Result<(), ()>;

    fn get_framebuffer_dimensions(&self) -> (u32, u32);
}

pub struct FlutterOutput<B: FlutterOutputBackend + Send + Sync + 'static> {
    engine: FlutterEngine,
    engine_handler: Arc<SmithayFlutterHandler>,
    backend: Arc<B>,
}

impl<B: FlutterOutputBackend + Send + Sync + 'static> Clone for FlutterOutput<B> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            engine_handler: self.engine_handler.clone(),
            backend: self.backend.clone(),
        }
    }
}

fn create_output<B>(
    backend: Arc<B>,
    options: &mut FlutterEngineOptions,
    keyboard: Arc<Mutex<KeyboardManager>>,
) -> (Parker, FlutterOutput<B>)
where
    B: FlutterOutputBackend + Send + Sync + 'static,
{
    let (resource_context, display) = unsafe {
        backend.make_current().expect("Invalid backend context");

        let display = WrappedDisplay::get_current();
        let resource_context = WrappedContext::create_context();
        display.clear_current();
        (resource_context, display)
    };

    let parker = Parker::new();
    let unparker = parker.unparker().clone();

    let engine_handler = Arc::new(SmithayFlutterHandler {
        backend: Arc::downgrade(&backend) as _,
        display,
        resource_context,
        unparker,
    });

    let engine = FlutterEngine::new(
        Arc::downgrade(&engine_handler) as _,
        options.assets_path.clone(),
    );

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
            engine_handler,
            backend,
        },
    )
}

fn run_output<B>(parker: Parker, output: FlutterOutput<B>, options: &FlutterEngineOptions)
where
    B: FlutterOutputBackend + Send + Sync + 'static,
{
    let (width, height) = output.backend.get_framebuffer_dimensions();

    output
        .engine
        .run(&options.arguments)
        .expect("Failed to start engine");

    output
        .engine
        .send_window_metrics_event(width as usize, height as usize, height as f64 / 1080.0);

    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::SeqCst) {
        output.engine.execute_platform_tasks();
        parker.park();
    }
}

impl<B: FlutterOutputBackend + Send + Sync + 'static> FlutterOutput<B> {
    pub(crate) fn new(
        backend: Arc<B>,
        options: FlutterEngineOptions,
        keyboard: Arc<Mutex<KeyboardManager>>,
    ) -> Self {
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
                run_output(parker, output, &options);
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
