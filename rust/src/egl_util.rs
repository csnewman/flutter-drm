use core::{mem, ptr};
use log::{debug, info};

use smithay::backend::egl::context::PixelFormatRequirements;
use smithay::backend::egl::{ffi, native};
use smithay::backend::graphics::{PixelFormat, SwapBuffersError};
use std::cell::Cell;
use std::ffi::{c_void, CStr, CString};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_int;

pub struct WrappedDisplay(ffi::egl::types::EGLDisplay);

unsafe impl Send for WrappedDisplay {}

unsafe impl Sync for WrappedDisplay {}

impl Clone for WrappedDisplay {
    fn clone(&self) -> Self {
        WrappedDisplay { 0: self.0 }
    }
}

impl WrappedDisplay {
    pub unsafe fn get_current() -> Self {
        let display = ffi::egl::GetCurrentDisplay();

        if display == ptr::null() {
            panic!("Failed to fetch display");
        }

        info!("Current display was {:?}", display);
        WrappedDisplay(display)
    }

    pub unsafe fn clear_current(&self) {
        let _ret = ffi::egl::MakeCurrent(self.0, ptr::null(), ptr::null(), ptr::null());
    }
}

pub struct WrappedContext {
    display: ffi::egl::types::EGLDisplay,
    context: ffi::egl::types::EGLContext,
    surface_attributes: Vec<c_int>,
    config_id: ffi::egl::types::EGLConfig,
    pixel_format: PixelFormat,
}

unsafe impl Send for WrappedContext {}

unsafe impl Sync for WrappedContext {}

impl Clone for WrappedContext {
    fn clone(&self) -> Self {
        WrappedContext {
            display: self.display,
            context: self.context,
            surface_attributes: self.surface_attributes.clone(),
            config_id: self.config_id.clone(),
            pixel_format: self.pixel_format.clone(),
        }
    }
}

impl WrappedContext {
    pub unsafe fn create_context() -> WrappedContext {
        debug!("Trying to initialize EGL with OpenGLES 3.0");

        let old_context = ffi::egl::GetCurrentContext();
        info!("Current context was {:?}", old_context);

        let display = ffi::egl::GetCurrentDisplay();
        info!("Current display was {:?}", display);

        create_context_inner((3, 0), old_context, display)
        //    attributes.version = Some((3, 0));
        //    match EGLContext::<B, N>::new_internal(ptr, attributes, reqs, log.clone()) {
        //        Ok(x) => return Ok(x),
        //        Err(err) => {
        //            warn!(log, "EGL OpenGLES 3.0 Initialization failed with {}", err);
        //            debug!(log, "Trying to initialize EGL with OpenGLES 2.0");
        //            attributes.version = Some((2, 0));
        //            return EGLContext::<B, N>::new_internal(ptr, attributes, reqs, log);
        //        }
        //    }
    }

    pub unsafe fn create_context_dir(
        share_context: ffi::egl::types::EGLContext,
        display: ffi::egl::types::EGLDisplay,
    ) -> WrappedContext {
        debug!("Trying to initialize EGL with OpenGLES 3.0");

        create_context_inner((3, 0), share_context, display)
        //    attributes.version = Some((3, 0));
        //    match EGLContext::<B, N>::new_internal(ptr, attributes, reqs, log.clone()) {
        //        Ok(x) => return Ok(x),
        //        Err(err) => {
        //            warn!(log, "EGL OpenGLES 3.0 Initialization failed with {}", err);
        //            debug!(log, "Trying to initialize EGL with OpenGLES 2.0");
        //            attributes.version = Some((2, 0));
        //            return EGLContext::<B, N>::new_internal(ptr, attributes, reqs, log);
        //        }
        //    }
    }

    pub unsafe fn make_current(&self) -> bool {
        let ret = ffi::egl::MakeCurrent(self.display, ptr::null(), ptr::null(), self.context);
        ret == 1
    }

    pub unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let addr = CString::new(symbol.as_bytes()).unwrap();
        let addr = addr.as_ptr();
        ffi::egl::GetProcAddress(addr) as *const _
    }

    pub fn is_current(&self) -> bool {
        unsafe { ffi::egl::GetCurrentContext() == self.context }
    }

    pub fn create_surface<N>(&self, native: N) -> WrappedSurface<N>
    where
        N: native::NativeSurface,
    {
        WrappedSurface::create(
            self.context,
            self.display,
            native,
            self.config_id,
            &self.surface_attributes,
        )
    }
}

unsafe fn create_context_inner(
    version: (u8, u8),
    share_context: ffi::egl::types::EGLContext,
    display: ffi::egl::types::EGLDisplay,
) -> WrappedContext {
    let reqs: PixelFormatRequirements = Default::default();

    let egl_version = {
        let mut major = mem::MaybeUninit::uninit();
        let mut minor = mem::MaybeUninit::uninit();

        if ffi::egl::Initialize(display, major.as_mut_ptr(), minor.as_mut_ptr()) == 0 {
            panic!("Display reinit failed");
        }

        let major = major.assume_init();
        let minor = minor.assume_init();

        info!("EGL Version: {:?}", (major, minor));

        (major, minor)
    };

    let extensions = if egl_version >= (1, 2) {
        let p = CStr::from_ptr(ffi::egl::QueryString(display, ffi::egl::EXTENSIONS as i32));
        let list = String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| String::new());
        list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
    } else {
        vec![]
    };

    info!("EGL Extensions: {:?}", extensions);

    if egl_version >= (1, 2) && ffi::egl::BindAPI(ffi::egl::OPENGL_ES_API) == 0 {
        panic!("OpenGLES not supported by the underlying EGL implementation");
    }

    let descriptor = {
        let mut out: Vec<c_int> = Vec::with_capacity(37);

        if egl_version >= (1, 2) {
            debug!("Setting COLOR_BUFFER_TYPE to RGB_BUFFER");
            out.push(ffi::egl::COLOR_BUFFER_TYPE as c_int);
            out.push(ffi::egl::RGB_BUFFER as c_int);
        }

        debug!("Setting SURFACE_TYPE to WINDOW");

        out.push(ffi::egl::SURFACE_TYPE as c_int);
        // TODO: Some versions of Mesa report a BAD_ATTRIBUTE error
        // if we ask for PBUFFER_BIT as well as WINDOW_BIT
        out.push((ffi::egl::WINDOW_BIT) as c_int);

        match version {
            (3, _) => {
                if egl_version < (1, 3) {
                    panic!("OpenglES 3.* is not supported on EGL Versions lower then 1.3");
                }
                debug!("Setting RENDERABLE_TYPE to OPENGL_ES3");
                out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                out.push(ffi::egl::OPENGL_ES3_BIT as c_int);
                debug!("Setting CONFORMANT to OPENGL_ES3");
                out.push(ffi::egl::CONFORMANT as c_int);
                out.push(ffi::egl::OPENGL_ES3_BIT as c_int);
            }
            (2, _) => {
                if egl_version < (1, 3) {
                    panic!("OpenglES 2.* is not supported on EGL Versions lower then 1.3");
                }
                debug!("Setting RENDERABLE_TYPE to OPENGL_ES2");
                out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                out.push(ffi::egl::OPENGL_ES2_BIT as c_int);
                debug!("Setting CONFORMANT to OPENGL_ES2");
                out.push(ffi::egl::CONFORMANT as c_int);
                out.push(ffi::egl::OPENGL_ES2_BIT as c_int);
            }
            (_, _) => unreachable!(),
        };

        if let Some(hardware_accelerated) = reqs.hardware_accelerated {
            out.push(ffi::egl::CONFIG_CAVEAT as c_int);
            out.push(if hardware_accelerated {
                debug!("Setting CONFIG_CAVEAT to NONE");
                ffi::egl::NONE as c_int
            } else {
                debug!("Setting CONFIG_CAVEAT to SLOW_CONFIG");
                ffi::egl::SLOW_CONFIG as c_int
            });
        }

        if let Some(color) = reqs.color_bits {
            debug!("Setting RED_SIZE to {}", color / 3);
            out.push(ffi::egl::RED_SIZE as c_int);
            out.push((color / 3) as c_int);
            debug!(
                "Setting GREEN_SIZE to {}",
                color / 3 + if color % 3 != 0 { 1 } else { 0 }
            );
            out.push(ffi::egl::GREEN_SIZE as c_int);
            out.push((color / 3 + if color % 3 != 0 { 1 } else { 0 }) as c_int);
            debug!(
                "Setting BLUE_SIZE to {}",
                color / 3 + if color % 3 == 2 { 1 } else { 0 }
            );
            out.push(ffi::egl::BLUE_SIZE as c_int);
            out.push((color / 3 + if color % 3 == 2 { 1 } else { 0 }) as c_int);
        }

        if let Some(alpha) = reqs.alpha_bits {
            debug!("Setting ALPHA_SIZE to {}", alpha);
            out.push(ffi::egl::ALPHA_SIZE as c_int);
            out.push(alpha as c_int);
        }

        if let Some(depth) = reqs.depth_bits {
            debug!("Setting DEPTH_SIZE to {}", depth);
            out.push(ffi::egl::DEPTH_SIZE as c_int);
            out.push(depth as c_int);
        }

        if let Some(stencil) = reqs.stencil_bits {
            debug!("Setting STENCIL_SIZE to {}", stencil);
            out.push(ffi::egl::STENCIL_SIZE as c_int);
            out.push(stencil as c_int);
        }

        if let Some(multisampling) = reqs.multisampling {
            debug!("Setting SAMPLES to {}", multisampling);
            out.push(ffi::egl::SAMPLES as c_int);
            out.push(multisampling as c_int);
        }

        if reqs.stereoscopy {
            panic!("Stereoscopy is currently unsupported (sorry!)");
        }

        out.push(ffi::egl::NONE as c_int);
        out
    };

    // calling `eglChooseConfig`

    let mut config_id = mem::MaybeUninit::uninit();
    let mut num_configs = mem::MaybeUninit::uninit();
    if ffi::egl::ChooseConfig(
        display,
        descriptor.as_ptr(),
        config_id.as_mut_ptr(),
        1,
        num_configs.as_mut_ptr(),
    ) == 0
    {
        panic!("Config failed");
    }
    let config_id = config_id.assume_init();
    let num_configs = num_configs.assume_init();

    if num_configs == 0 {
        panic!("No matching color format found");
    }

    // analyzing each config
    macro_rules! attrib {
        ($display:expr, $config:expr, $attr:expr) => {{
            let mut value = mem::MaybeUninit::uninit();
            let res = ffi::egl::GetConfigAttrib(
                $display,
                $config,
                $attr as ffi::egl::types::EGLint,
                value.as_mut_ptr(),
            );
            if res == 0 {
                panic!("Config failed");
            }
            value.assume_init()
        }};
    };

    let desc = PixelFormat {
        hardware_accelerated: attrib!(display, config_id, ffi::egl::CONFIG_CAVEAT)
            != ffi::egl::SLOW_CONFIG as i32,
        color_bits: attrib!(display, config_id, ffi::egl::RED_SIZE) as u8
            + attrib!(display, config_id, ffi::egl::BLUE_SIZE) as u8
            + attrib!(display, config_id, ffi::egl::GREEN_SIZE) as u8,
        alpha_bits: attrib!(display, config_id, ffi::egl::ALPHA_SIZE) as u8,
        depth_bits: attrib!(display, config_id, ffi::egl::DEPTH_SIZE) as u8,
        stencil_bits: attrib!(display, config_id, ffi::egl::STENCIL_SIZE) as u8,
        stereoscopy: false,
        double_buffer: true,
        multisampling: match attrib!(display, config_id, ffi::egl::SAMPLES) {
            0 | 1 => None,
            a => Some(a as u16),
        },
        srgb: false, // TODO: use EGL_KHR_gl_colorspace to know that
    };

    info!("Selected color format: {:?}", desc);

    let mut context_attributes = Vec::with_capacity(10);

    if egl_version >= (1, 5) || extensions.iter().any(|s| *s == "EGL_KHR_create_context") {
        debug!("Setting CONTEXT_MAJOR_VERSION to {}", version.0);
        context_attributes.push(ffi::egl::CONTEXT_MAJOR_VERSION as i32);
        context_attributes.push(version.0 as i32);
        debug!("Setting CONTEXT_MINOR_VERSION to {}", version.1);
        context_attributes.push(ffi::egl::CONTEXT_MINOR_VERSION as i32);
        context_attributes.push(version.1 as i32);

        context_attributes.push(ffi::egl::CONTEXT_FLAGS_KHR as i32);
        context_attributes.push(0);
    } else if egl_version >= (1, 3) {
        debug!("Setting CONTEXT_CLIENT_VERSION to {}", version.0);
        context_attributes.push(ffi::egl::CONTEXT_CLIENT_VERSION as i32);
        context_attributes.push(version.0 as i32);
    }

    context_attributes.push(ffi::egl::NONE as i32);

    debug!("Creating EGL context...");

    let context = ffi::egl::CreateContext(
        display,
        config_id,
        share_context,
        context_attributes.as_ptr(),
    );

    if context.is_null() {
        match ffi::egl::GetError() as u32 {
            ffi::egl::BAD_ATTRIBUTE => panic!("Creation failed"),
            err_no => panic!("Unknown error {}", err_no),
        }
    }
    debug!("EGL context successfully created");

    let surface_attributes = {
        let mut out: Vec<c_int> = Vec::with_capacity(3);

        match reqs.double_buffer {
            Some(true) => {
                debug!("Setting RENDER_BUFFER to BACK_BUFFER");
                out.push(ffi::egl::RENDER_BUFFER as c_int);
                out.push(ffi::egl::BACK_BUFFER as c_int);
            }
            Some(false) => {
                debug!("Setting RENDER_BUFFER to SINGLE_BUFFER");
                out.push(ffi::egl::RENDER_BUFFER as c_int);
                out.push(ffi::egl::SINGLE_BUFFER as c_int);
            }
            None => {}
        }

        out.push(ffi::egl::NONE as i32);
        out
    };
    info!("EGL context created");

    WrappedContext {
        display,
        context,
        surface_attributes,
        config_id,
        pixel_format: desc,
    }
}

pub struct WrappedSurface<N: native::NativeSurface> {
    context: ffi::egl::types::EGLContext,
    display: ffi::egl::types::EGLDisplay,
    native: N,
    surface: Cell<ffi::egl::types::EGLSurface>,
    config_id: ffi::egl::types::EGLConfig,
    surface_attributes: Vec<c_int>,
}

impl<N: native::NativeSurface> Deref for WrappedSurface<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.native
    }
}

impl<N: native::NativeSurface> DerefMut for WrappedSurface<N> {
    fn deref_mut(&mut self) -> &mut N {
        &mut self.native
    }
}

unsafe impl<N: native::NativeSurface> Send for WrappedSurface<N> {}

unsafe impl<N: native::NativeSurface> Sync for WrappedSurface<N> {}

impl<N: native::NativeSurface> WrappedSurface<N> {
    pub fn create(
        context: ffi::egl::types::EGLContext,
        display: ffi::egl::types::EGLDisplay,
        native: N,
        config_id: ffi::egl::types::EGLConfig,
        surface_attributes: &Vec<c_int>,
    ) -> Self {
        let surface = unsafe {
            ffi::egl::CreateWindowSurface(
                display,
                config_id,
                native.ptr(),
                surface_attributes.as_ptr(),
            )
        };

        if surface.is_null() {
            panic!("Surface creation failed");
        }

        WrappedSurface {
            context,
            display,
            native,
            surface: Cell::new(surface),
            config_id,
            surface_attributes: surface_attributes.clone(),
        }
    }

    pub fn swap_buffers(&self) -> ::std::result::Result<(), SwapBuffersError> {
        let surface = self.surface.get();

        if !surface.is_null() {
            let ret =
                unsafe { ffi::egl::SwapBuffers(self.display as *const _, surface as *const _) };

            if ret == 0 {
                match unsafe { ffi::egl::GetError() } as u32 {
                    ffi::egl::CONTEXT_LOST => return Err(SwapBuffersError::ContextLost),
                    err => return Err(SwapBuffersError::Unknown(err)),
                };
            } else {
                self.native.swap_buffers()?;
            }
        };

        if self.native.needs_recreation() || surface.is_null() {
            self.native.recreate();
            self.surface.set(unsafe {
                ffi::egl::CreateWindowSurface(
                    self.display,
                    self.config_id,
                    self.native.ptr(),
                    self.surface_attributes.as_ptr(),
                )
            });
        }

        Ok(())
    }

    pub unsafe fn make_current(&self) -> ::std::result::Result<(), SwapBuffersError> {
        let ret = ffi::egl::MakeCurrent(
            self.display as *const _,
            self.surface.get() as *const _,
            self.surface.get() as *const _,
            self.context as *const _,
        );

        if ret == 0 {
            match ffi::egl::GetError() as u32 {
                ffi::egl::CONTEXT_LOST => Err(SwapBuffersError::ContextLost),
                err => panic!("eglMakeCurrent failed (eglGetError returned 0x{:x})", err),
            }
        } else {
            Ok(())
        }
    }

    pub fn is_current(&self) -> bool {
        unsafe {
            ffi::egl::GetCurrentSurface(ffi::egl::DRAW as _) == self.surface.get() as *const _
                && ffi::egl::GetCurrentSurface(ffi::egl::READ as _)
                    == self.surface.get() as *const _
        }
    }
}

impl<N: native::NativeSurface> Drop for WrappedSurface<N> {
    fn drop(&mut self) {
        unsafe {
            ffi::egl::DestroySurface(self.display as *const _, self.surface.get() as *const _);
        }
    }
}
