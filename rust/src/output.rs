use crate::egl_util::{WrappedContext, WrappedDisplay};
use crate::handler::SmithayFlutterHandler;
use flutter_engine::FlutterEngine;
use log::debug;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{panic, thread};

pub trait FlutterOutputBackend {
    fn swap_buffers(&self) -> Result<(), ()>;

    fn make_current(&self) -> Result<(), ()>;

    fn get_framebuffer_dimensions(&self) -> (u32, u32);
}

pub struct FlutterOutput {
    engine: FlutterEngine,
    engine_handler: Arc<SmithayFlutterHandler>,
    backend: Arc<dyn FlutterOutputBackend + Send + Sync>,
}

impl Clone for FlutterOutput {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            engine_handler: self.engine_handler.clone(),
            backend: self.backend.clone(),
        }
    }
}

fn create_output(backend: Arc<dyn FlutterOutputBackend + Send + Sync>) -> FlutterOutput {
    let (resource_context, display) = unsafe {
        backend.make_current().expect("Invalid backend context");

        let display = WrappedDisplay::get_current();
        let resource_context = WrappedContext::create_context();
        display.clear_current();
        (resource_context, display)
    };

    let engine_handler = Arc::new(SmithayFlutterHandler {
        backend: Arc::downgrade(&backend) as _,
        display,
        resource_context,
    });

    let engine = FlutterEngine::new(Arc::downgrade(&engine_handler) as _);

    FlutterOutput {
        engine,
        engine_handler,
        backend,
    }
}

fn run_output(output: FlutterOutput) {
    let (width, height) = output.backend.get_framebuffer_dimensions();

    output
        .engine
        .run(
            "flutter_assets/".to_string(),
            "icudtl.dat".to_string(),
            Vec::new(),
        )
        .expect("Failed to start engine");

    output
        .engine
        .send_window_metrics_event(width as i32, height as i32, (height as f64) / 1080.0);

    let running = Arc::new(AtomicBool::new(true));

    while running.load(Ordering::SeqCst) {
        //                input.dispatch_new_events().unwrap();

        output.engine.execute_platform_tasks();

        thread::sleep(Duration::from_secs(1));

        //                if event_loop
        //                    .dispatch(Some(::std::time::Duration::from_millis(16)), &mut ())
        //                    .is_err()
        //                {
        //                    running.store(false, Ordering::SeqCst);
        //                }
    }
}

impl FlutterOutput {
    pub(crate) fn new(backend: Arc<dyn FlutterOutputBackend + Send + Sync>) -> Self {
        debug!("Creating new flutter output");

        let (send, recv) = mpsc::channel();
        thread::spawn(move || {
            let panic_sender = send.clone();
            let mut has_sent = false;
            let result = panic::catch_unwind(AssertUnwindSafe(move || {
                let output = create_output(backend);
                send.send(Ok(output.clone())).unwrap();
                has_sent = true;
                run_output(output);
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
}
