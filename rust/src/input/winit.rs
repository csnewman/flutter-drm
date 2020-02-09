use crate::input::keyboard::KeyboardManager;
use parking_lot::Mutex;
use smithay::backend::input::{InputHandler, KeyboardKeyEvent, Seat, UnusedEvent};
use smithay::backend::winit::{
    WinitInputBackend, WinitKeyboardInputEvent, WinitMouseInputEvent, WinitMouseMovedEvent,
    WinitMouseWheelEvent, WinitTouchCancelledEvent, WinitTouchEndedEvent, WinitTouchMovedEvent,
    WinitTouchStartedEvent,
};
use std::sync::Arc;

pub struct WinitInputHandler {
    keyboard: Arc<Mutex<KeyboardManager>>,
}

impl WinitInputHandler {
    pub fn new(keyboard: Arc<Mutex<KeyboardManager>>) -> Self {
        Self { keyboard }
    }
}

impl InputHandler<WinitInputBackend> for WinitInputHandler {
    fn on_seat_created(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_seat_destroyed(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_seat_changed(&mut self, seat: &Seat) {
        // currently we just create a single static one
    }

    fn on_keyboard_key(&mut self, seat: &Seat, event: WinitKeyboardInputEvent) {
        let mut keyboard = self.keyboard.lock();

        // TODO: Select keyboard layout based on winit settings
        keyboard.select_layout();

        // Send key press
        let keycode = event.key_code();
        let state = event.state();
        keyboard.key(keycode, state);
    }

    fn on_pointer_move(&mut self, seat: &Seat, event: UnusedEvent) {
        // Not supported by winit
    }

    fn on_pointer_move_absolute(&mut self, seat: &Seat, event: WinitMouseMovedEvent) {
        // TODO: Implement pointer support
    }

    fn on_pointer_button(&mut self, seat: &Seat, event: WinitMouseInputEvent) {
        // TODO: Implement pointer support
    }

    fn on_pointer_axis(&mut self, seat: &Seat, event: WinitMouseWheelEvent) {
        // TODO: Implement pointer support
    }

    fn on_touch_down(&mut self, seat: &Seat, event: WinitTouchStartedEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_motion(&mut self, seat: &Seat, event: WinitTouchMovedEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_up(&mut self, seat: &Seat, event: WinitTouchEndedEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_cancel(&mut self, seat: &Seat, event: WinitTouchCancelledEvent) {
        // TODO: Implement touch support
    }

    fn on_touch_frame(&mut self, seat: &Seat, event: UnusedEvent) {
        // Not supported by winit
    }

    fn on_input_config_changed(&mut self, config: &mut ()) {
        // Not supported by winit
    }
}
