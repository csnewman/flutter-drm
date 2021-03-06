use crate::input::libinput::LibInputHandler;
use smithay::backend::drm::egl::{EglDevice, EglSurface};
use smithay::backend::drm::gbm::{egl::Gbm as EglGbmBackend, GbmDevice, GbmSurface};
use smithay::backend::drm::legacy::LegacyDrmDevice;
use smithay::backend::drm::{device_bind, Device, DeviceHandler, Surface};
use smithay::backend::egl::EGLContext;
use smithay::backend::libinput::{libinput_bind, LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::session::auto::{auto_session_bind, AutoId, AutoSession, BoundAutoSession};
use smithay::backend::session::{notify_multiplexer, AsSessionObserver, Session, SessionNotifier};
use smithay::backend::udev::{udev_backend_bind, UdevBackend, UdevHandler};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error as IoError;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::rc::Rc;

use std::sync::Arc;

use smithay::reexports::calloop::{
    generic::{Generic, SourceFd},
    LoopHandle, Source,
};

use smithay::reexports::{
    drm::control::{
        connector::{Info as ConnectorInfo, State as ConnectorState},
        crtc,
        encoder::Info as EncoderInfo,
    },
    input::Libinput,
    nix::{fcntl::OFlag, sys::stat::dev_t},
};

use log::{error, info, trace};

use crate::egl_util::{WrappedContext, WrappedSurface};

use crate::input::keyboard::KeyboardManager;
use crate::output::{FlutterEngineOptions, FlutterOutput, FlutterOutputBackend};
use crate::{EngineWeakCollection, FlutterDrmManager};
use parking_lot::Mutex;
use smithay::backend::input::InputBackend;

pub struct SessionFd(RawFd);

impl AsRawFd for SessionFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

type RenderDevice =
    EglDevice<EglGbmBackend<LegacyDrmDevice<SessionFd>>, GbmDevice<LegacyDrmDevice<SessionFd>>>;
type RenderSurface =
    EglSurface<EglGbmBackend<LegacyDrmDevice<SessionFd>>, GbmDevice<LegacyDrmDevice<SessionFd>>>;
type WrappedRenderSurface = WrappedSurface<GbmSurface<LegacyDrmDevice<SessionFd>>>;

struct DrmOutputBackend {
    surface: WrappedRenderSurface,
}

impl FlutterOutputBackend for DrmOutputBackend {
    fn swap_buffers(&self) -> Result<(), ()> {
        self.surface.swap_buffers().map_err(|_| ())
    }

    fn make_current(&self) -> Result<(), ()> {
        unsafe { self.surface.make_current().map_err(|_| ()) }
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let (w, h) = self
            .surface
            .pending_mode()
            .map(|mode| mode.size())
            .unwrap_or((1, 1));
        (w as u32, h as u32)
    }
}

pub trait UdevOutputManagerHandler {
    fn should_use_gpu(&self, path: PathBuf) -> bool;

    fn configure_output(&self) -> Option<FlutterEngineOptions>;
}

pub struct UdevOutputManager<S: SessionNotifier + 'static> {
    engines: EngineWeakCollection,
    keyboard: Arc<Mutex<KeyboardManager>>,
    session: AutoSession,
    udev_session_id: AutoId,
    seat: String,
    libinput_session_id: AutoId,
    libinput_event_source: Source<Generic<SourceFd<LibinputInputBackend>>>,
    session_event_source: BoundAutoSession,
    udev_event_source: Source<Generic<SourceFd<UdevBackend<UdevHandlerImpl<S, ()>>>>>,
}

pub fn new_udev(
    manager: &FlutterDrmManager,
    handler: Arc<dyn UdevOutputManagerHandler>,
) -> UdevOutputManager<impl SessionNotifier + 'static> {
    let engines = EngineWeakCollection::new();
    let keyboard = Arc::new(Mutex::new(KeyboardManager::new(engines.clone())));

    // Init session
    let (session, mut notifier) = AutoSession::new(None).ok_or(()).unwrap();
    let (udev_observer, udev_notifier) = notify_multiplexer();
    let udev_session_id = notifier.register(udev_observer);

    // Initialize the udev backend
    let seat = session.seat();

    // TODO: Find primary gpu
    //    let primary_gpu = primary_gpu(&context, &seat).unwrap_or_default();
    //    if let None = primary_gpu {
    //        panic!("No primary gpu detected");
    //    }

    let udev_backend = UdevBackend::new(
        UdevHandlerImpl {
            engines: engines.clone(),
            keyboard: keyboard.clone(),
            handler,
            session: session.clone(),
            backends: HashMap::new(),
            loop_handle: manager.event_loop.handle(),
            notifier: udev_notifier,
        },
        seat.clone(),
        None,
    )
    .map_err(|_| ())
    .unwrap();

    // Initialize libinput backend
    let mut libinput_context =
        Libinput::new_with_udev::<LibinputSessionInterface<AutoSession>>(session.clone().into());
    let libinput_session_id = notifier.register(libinput_context.observer());
    libinput_context.udev_assign_seat(&seat).unwrap();
    let mut libinput_backend = LibinputInputBackend::new(libinput_context, None);
    libinput_backend.set_handler(LibInputHandler::new(keyboard.clone(), session.clone()));

    // Bind all our objects that get driven by the event loop
    let libinput_event_source = libinput_bind(libinput_backend, manager.event_loop.handle())
        .map_err(|e| -> IoError { e.into() })
        .unwrap();
    let session_event_source = auto_session_bind(notifier, &manager.event_loop.handle())
        .map_err(|(e, _)| e)
        .unwrap();
    let udev_event_source = udev_backend_bind(udev_backend, &manager.event_loop.handle())
        .map_err(|e| -> IoError { e.into() })
        .unwrap();

    UdevOutputManager {
        engines,
        keyboard,
        session,
        udev_session_id,
        seat,
        libinput_session_id,
        libinput_event_source,
        session_event_source,
        udev_event_source,
    }
}

impl<S: SessionNotifier + 'static> UdevOutputManager<S> {
    pub fn cleanup(self) {
        let mut notifier = self.session_event_source.unbind();
        notifier.unregister(self.libinput_session_id);
        notifier.unregister(self.udev_session_id);

        self.libinput_event_source.remove();
        self.udev_event_source.remove();
    }
}

struct UdevHandlerImpl<S: SessionNotifier, Data: 'static> {
    engines: EngineWeakCollection,
    keyboard: Arc<Mutex<KeyboardManager>>,
    handler: Arc<dyn UdevOutputManagerHandler>,
    session: AutoSession,
    backends: HashMap<
        dev_t,
        (
            S::Id,
            Source<Generic<SourceFd<RenderDevice>>>,
            Rc<RefCell<HashMap<crtc::Handle, FlutterOutput>>>,
        ),
    >,
    loop_handle: LoopHandle<Data>,
    notifier: S,
}

impl<S: SessionNotifier, Data: 'static> UdevHandlerImpl<S, Data> {
    pub fn scan_connectors(
        &self,
        device: &mut RenderDevice,
    ) -> HashMap<crtc::Handle, FlutterOutput> {
        // Get a set of all modesetting resource handles (excluding planes)
        let res_handles = device.resource_handles().unwrap();

        // Use first connected connector
        let connector_infos: Vec<ConnectorInfo> = res_handles
            .connectors()
            .iter()
            .map(|conn| device.get_connector_info(*conn).unwrap())
            .filter(|conn| conn.state() == ConnectorState::Connected)
            .inspect(|conn| info!("Connected: {:?}", conn.interface()))
            .collect();

        let mut backends = HashMap::new();

        // very naive way of finding good crtc/encoder/connector combinations.
        for connector_info in connector_infos {
            let encoder_infos = connector_info
                .encoders()
                .iter()
                .filter_map(|e| *e)
                .flat_map(|encoder_handle| device.get_encoder_info(encoder_handle))
                .collect::<Vec<EncoderInfo>>();
            for encoder_info in encoder_infos {
                'inner: for crtc in res_handles.filter_crtcs(encoder_info.possible_crtcs()) {
                    if !backends.contains_key(&crtc) {
                        // device.get_crtc_info()
                        // let info = device.resource_info::<crtc::Info>(crtc);
                        // info!("CRTC Info: {:?}", info);

                        let options = match self.handler.configure_output() {
                            Some(opts) => opts,
                            None => continue 'inner,
                        };

                        // Create new egl context for rendering
                        let device_context = device.get_context();
                        let raw_context = device_context.get_raw_context();
                        let raw_display = device_context.get_raw_display();
                        let render_context = unsafe {
                            WrappedContext::create_context_dir(*raw_context, *raw_display)
                        };

                        // Create new egl surface to render to
                        let surface = EGLContext::borrow_mut(&device_context)
                            .create_surface(crtc)
                            .unwrap();
                        let surface = render_context.create_surface(surface);

                        // Create output
                        let backend = DrmOutputBackend { surface };
                        let output = FlutterOutput::new(backend, options, self.keyboard.clone());
                        let engine = output.engine();
                        self.engines.add(engine.downgrade());

                        backends.insert(crtc, output);
                        break;
                    }
                }
            }
        }

        backends
    }
}

impl<S: SessionNotifier, Data: 'static> UdevHandler for UdevHandlerImpl<S, Data> {
    fn device_added(&mut self, _device: dev_t, path: PathBuf) {
        if !self.handler.should_use_gpu(path.canonicalize().unwrap()) {
            return;
        }

        info!("Device added: {:?}", path.canonicalize().unwrap());

        // Try to open the device
        if let Some(mut device) = self
            .session
            .open(
                &path,
                OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NOCTTY | OFlag::O_NONBLOCK,
            )
            .ok()
            .and_then(|fd| LegacyDrmDevice::new(SessionFd(fd), None).ok())
            .and_then(|drm| GbmDevice::new(drm, None).ok())
            .and_then(|gbm| EglDevice::new(gbm, None).ok())
        {
            let backends = Rc::new(RefCell::new(self.scan_connectors(&mut device)));

            // Set the handler.
            device.set_handler(DrmHandlerImpl {
                backends: backends.clone(),
            });

            let device_session_id = self.notifier.register(device.observer());
            let dev_id = device.device_id();
            let event_source = device_bind(&self.loop_handle, device)
                .map_err(|e| -> IoError { e.into() })
                .unwrap();

            self.backends
                .insert(dev_id, (device_session_id, event_source, backends));
        }
    }

    fn device_changed(&mut self, _device: dev_t) {
        //quick and dirty, just re-init all backends
        //        if let Some((_, ref mut evt_source, ref backends)) = self.backends.get_mut(&device) {
        //            let source = evt_source.clone_inner();
        //            let mut evented = source.borrow_mut();
        //            let mut backends = backends.borrow_mut();
        //            let new_backends = UdevHandlerImpl::<S, Data>::scan_connectors(
        //                &mut (*evented).0,
        //                &self.logger,
        //            );
        //            *backends = new_backends;
        //        }

        error!("Device change not implemented");
    }

    fn device_removed(&mut self, _device: dev_t) {
        error!("Device remove not implemented");

        // drop the backends on this side
        //        if let Some((id, evt_source, renderers)) = self.backends.remove(&device) {
        //            // drop surfaces
        //            renderers.borrow_mut().clear();
        //            debug!("Surfaces dropped");
        //
        //            let device = Rc::try_unwrap(evt_source.remove().unwrap())
        //                .map_err(|_| "This should not happend")
        //                .unwrap()
        //                .into_inner()
        //                .0;
        //
        //            // TODO: Exit if primary gpu was disconnected
        //            if device.dev_path().and_then(|path| path.canonicalize().ok()) == self.primary_gpu {
        //                unimplemented!();
        //            }
        //
        //            self.notifier.unregister(id);
        //            debug!("Dropping device");
        //        }
    }
}

pub struct DrmHandlerImpl {
    backends: Rc<RefCell<HashMap<crtc::Handle, FlutterOutput>>>,
}

impl DeviceHandler for DrmHandlerImpl {
    type Device = RenderDevice;

    fn vblank(&mut self, crtc: crtc::Handle) {
        if let Some(_engine) = self.backends.borrow().get(&crtc) {
            trace!("vblank");
            //            let now = Instant::now();
            //            _engine
            //                .engine()
            //                .notify_vsync(now + Duration::from_millis(16), now + Duration::from_millis(32));
        }
    }

    fn error(&mut self, error: <RenderSurface as Surface>::Error) {
        error!("Device handler error: {:?}", error);
    }
}
