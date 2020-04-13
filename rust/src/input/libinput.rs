use crate::input::keyboard::KeyboardManager;
use parking_lot::Mutex;
use smithay::backend::input::{InputHandler, KeyState, KeyboardKeyEvent, Seat};
use smithay::backend::libinput::LibinputInputBackend;
use smithay::reexports::input as libinput;
use smithay::reexports::input::event;
use std::sync::Arc;

use log::error;
use smithay::backend::session::auto::AutoSession;
use smithay::backend::session::Session;

pub struct LibInputHandler {
    keyboard: Arc<Mutex<KeyboardManager>>,
    session: AutoSession,
}

impl LibInputHandler {
    pub fn new(keyboard: Arc<Mutex<KeyboardManager>>, session: AutoSession) -> Self {
        Self { keyboard, session }
    }
}

impl InputHandler<LibinputInputBackend> for LibInputHandler {
    fn on_seat_created(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_seat_destroyed(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_seat_changed(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_keyboard_key(&mut self, seat: &Seat, event: event::keyboard::KeyboardKeyEvent) {
        let mut keyboard = self.keyboard.lock();

        // TODO: Select keyboard layout
        // let device = event.device();
        keyboard.select_layout();

        // Send key press
        let keycode = event.key_code();
        let state = event.state();
        keyboard.key(keycode, state);

        // error!("keycode: {}", keycode);
        if state == KeyState::Pressed {
            if keycode >= 59 && keycode <= 70 {
                let vt = keycode - 59 + 1;
                error!("vt switch: {}", vt);
                self.session.change_vt(vt as i32);
            }
        }
    }

    fn on_pointer_move(&mut self, seat: &Seat, event: event::pointer::PointerMotionEvent) {
        // TODO: Implement pointer support
    }

    fn on_pointer_move_absolute(
        &mut self,
        seat: &Seat,
        event: event::pointer::PointerMotionAbsoluteEvent,
    ) {
        // TODO: Implement pointer support
    }

    fn on_pointer_button(&mut self, seat: &Seat, event: event::pointer::PointerButtonEvent) {
        // TODO: Implement pointer support
    }

    fn on_pointer_axis(&mut self, seat: &Seat, event: event::pointer::PointerAxisEvent) {
        // TODO: Implement pointer support
    }

    fn on_touch_down(&mut self, seat: &Seat, event: event::touch::TouchDownEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_motion(&mut self, seat: &Seat, event: event::touch::TouchMotionEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_up(&mut self, seat: &Seat, event: event::touch::TouchUpEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_cancel(&mut self, seat: &Seat, event: event::touch::TouchCancelEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_frame(&mut self, seat: &Seat, event: event::touch::TouchFrameEvent) {
        // TODO: Implement touch support
    }

    fn on_input_config_changed(&mut self, config: &mut [libinput::Device]) {
        let mut keyboards = Vec::new();

        for device in config {
            if device.has_capability(libinput::DeviceCapability::Keyboard) {
                keyboards.push(device.clone());
            }
        }

        self.keyboard.lock().update_devices(keyboards);
    }
}
