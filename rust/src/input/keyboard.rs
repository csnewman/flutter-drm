use crate::input::glfw;
use crate::EngineWeakCollection;
use crossbeam::channel;
use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};
use flutter_engine::FlutterEngineWeakRef;
use flutter_plugins::keyevent::{KeyAction, KeyActionType, KeyEventPlugin};
use flutter_plugins::textinput::TextInputPlugin;
use log::debug;
use log::info;
use smithay::backend::input::KeyState;
use smithay::reexports::input as libinput;
use std::thread;
use std::time::{Duration, Instant};
use xkbcommon::xkb;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyboardConfig {
    /// The rules file to use.
    ///
    /// The rules file describes how to interpret the values of the model, layout, variant and
    /// options fields.
    pub rules: String,
    /// The keyboard model by which to interpret keycodes and LEDs.
    pub model: String,
    /// A comma separated list of layouts (languages) to include in the keymap.
    pub layout: String,
    /// A comma separated list of variants, one per layout, which may modify or augment the
    /// respective layout in various ways.
    pub variant: String,
    /// A comma separated list of options, through which the user specifies non-layout related
    /// preferences, like which key combinations are used for switching layouts, or which key is the
    /// Compose key.
    pub options: Option<String>,
    /// The rate at which to repeat key press events.
    pub rate: i32,
    /// The delay after which key press events should be repeated when being held.
    pub delay: i32,
}

enum KeyRepeatAction {
    Pressed(KeyRepeatInfo),
    Released(u32),
    Stop,
}

struct KeyRepeatInfo {
    code: u32,
    state: xkb::State,
}

unsafe impl Send for KeyRepeatInfo {}

fn key_repeater_thread(repeat_recv: Receiver<KeyRepeatAction>, engines: EngineWeakCollection,
                       textinput: Arc<Mutex<Option<FlutterEngineWeakRef>>>) {
    let rate = 50;
    let delay = 1000;

    let mut repeat = None;

    'outer: loop {
        // Wait until we have a key press
        while repeat.is_none() {
            match repeat_recv.recv() {
                Ok(act) => match act {
                    KeyRepeatAction::Pressed(info) => {
                        repeat = Some(info);
                    }
                    _ => {}
                },
                Err(_) => return,
            };
        }

        let repeat_info = match repeat.as_ref() {
            None => {
                continue 'outer;
            }
            Some(info) => info,
        };

        let mut next_send = Instant::now() + Duration::from_millis(delay);
        loop {
            let mut now = Instant::now();

            // Check if we should send a key now
            if next_send <= now {
                // TODO: Send repeat event
                info!("We should repeat {} now", repeat_info.code);
                key_event(
                    repeat_info.code,
                    KeyState::Pressed,
                    &repeat_info.state,
                    &engines,
                    &textinput
                );

                next_send = now + Duration::from_millis(rate);
                now = Instant::now();
            }

            // Ensure we don't try to sleep for a negative length of time, possible if system can't
            // keep up with key repeats
            let timeout = if next_send <= now {
                Duration::from_millis(0)
            } else {
                next_send - now
            };

            match repeat_recv.recv_timeout(timeout) {
                Ok(act) => {
                    match act {
                        KeyRepeatAction::Pressed(info) => {
                            // Ignore we have press same key again
                            if repeat_info.code != info.code {
                                repeat = Some(info);
                                continue 'outer;
                            }
                        }
                        KeyRepeatAction::Released(key) => {
                            // Ensure we have released the same key
                            if repeat_info.code == key {
                                repeat = None;
                                continue 'outer;
                            }
                        }
                        KeyRepeatAction::Stop => {
                            repeat = None;
                            continue 'outer;
                        }
                    }
                }
                Err(err) => match err {
                    RecvTimeoutError::Timeout => {}
                    RecvTimeoutError::Disconnected => return,
                },
            }
        }
    }
}

struct ActiveConfig {
    config: KeyboardConfig,
    keymap: xkb::Keymap,
    state: xkb::State,
}

pub struct KeyboardManager {
    context: xkb::Context,
    current_config: Option<ActiveConfig>,
    repeat_sender: Sender<KeyRepeatAction>,
    engines: EngineWeakCollection,
    devices: Vec<libinput::Device>,
    textinput: Arc<Mutex<Option<FlutterEngineWeakRef>>>,
}

unsafe impl Send for KeyboardManager {}

impl KeyboardManager {
    pub fn new(engines: EngineWeakCollection) -> Self {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

        let (repeat_sender, repeat_recv) = channel::unbounded();

        let textinput = Arc::new(Mutex::new(None));

        let engines_copy = engines.clone();
        let textinput_copy = textinput.clone();
        thread::Builder::new()
            .name("keyboard-keyrepeater".to_string())
            .spawn(move || key_repeater_thread(repeat_recv, engines_copy, textinput_copy))
            .expect("Failed to create key repeater thread");

        Self {
            context,
            current_config: None,
            repeat_sender,
            engines,
            devices: vec![],
            textinput,
        }
    }

    pub fn select_layout(&mut self) {
        if self.current_config.is_none() {
            // TODO: Allow keyboard configuration
            // For now we just force the default layout and repeat options
            let config = KeyboardConfig {
                rules: "".to_string(),
                model: "".to_string(),
                layout: "".to_string(),
                variant: "".to_string(),
                options: None,
                rate: 50,
                delay: 1000,
            };

            let keymap = xkb::Keymap::new_from_names(
                &self.context,
                &config.rules,
                &config.model,
                &config.layout,
                &config.variant,
                config.options.clone(),
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )
            .unwrap();
            let state = xkb::State::new(&keymap);

            // Stop current repeat, as config has changed
            self.repeat_sender.send(KeyRepeatAction::Stop).unwrap();

            self.current_config = Some(ActiveConfig {
                config,
                keymap,
                state,
            });
        }
    }

    pub fn key(&mut self, rawcode: u32, keystate: KeyState) {
        let scancode = rawcode + 8;

        let config = self.current_config.as_mut().expect("No layout is active");

        // Update state
        let direction = match keystate {
            KeyState::Pressed => xkb::KeyDirection::Down,
            KeyState::Released => xkb::KeyDirection::Up,
        };
        config.state.update_key(scancode, direction);

        // Dispatch key press
        key_event(rawcode, keystate, &mut config.state, &self.engines, &self.textinput);

        self.repeat_sender
            .send(match keystate {
                KeyState::Released => KeyRepeatAction::Released(rawcode),
                KeyState::Pressed => match config.keymap.key_repeats(scancode) {
                    true => KeyRepeatAction::Pressed(KeyRepeatInfo {
                        code: rawcode,
                        state: config.state.clone(),
                    }),
                    false => KeyRepeatAction::Stop,
                },
            })
            .unwrap();

        self.update_leds();
    }

    pub fn update_devices(&mut self, devices: Vec<libinput::Device>) {
        self.devices = devices;
        self.update_leds();
    }

    fn update_leds(&mut self) {
        let mut leds = libinput::Led::empty();

        if let Some(config) = self.current_config.as_ref() {
            if config
                .state
                .mod_name_is_active(xkb::MOD_NAME_CAPS, xkb::STATE_MODS_EFFECTIVE)
            {
                leds |= libinput::Led::CAPSLOCK;
            }
            if config
                .state
                .mod_name_is_active(xkb::MOD_NAME_NUM, xkb::STATE_MODS_EFFECTIVE)
            {
                leds |= libinput::Led::NUMLOCK;
            }
        }

        for device in &mut self.devices {
            device.led_update(leds);
        }
    }

    pub fn set_text_target(&mut self, engine: FlutterEngineWeakRef) {
        *self.textinput.lock() = Some(engine);
    }

    pub fn clear_text_target(&mut self, engine: FlutterEngineWeakRef) {
        let mut textinput = self.textinput.lock();
        if let Some(current) = textinput.take() {
            if !current.ptr_equal(engine) {
                *textinput = Some(current);
            }
        }
    }
}

fn key_event(
    rawcode: u32,
    keystate: KeyState,
    state: &xkb::State,
    engines: &EngineWeakCollection,
    textinput: &Arc<Mutex<Option<FlutterEngineWeakRef>>>,
) {
    // Offset the rawcode by 8, as the evdev XKB rules reflect X's
    // broken keycode system, which starts at 8.
    let scancode = rawcode + 8;
    let keycode = glfw::map_key(rawcode);
    let content = state.key_get_utf8(scancode);

    debug!(
        "key event scancode={} state={:?} keycode={}, content='{}'",
        scancode, keystate, keycode, content,
    );

    // Convert modifiers
    let shift = state.mod_name_is_active(xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE) as i32;
    let ctrl = state.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE) as i32;
    let alt = state.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE) as i32;
    let logo = state.mod_name_is_active(xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE) as i32;
    let caps = state.mod_name_is_active(xkb::MOD_NAME_CAPS, xkb::STATE_MODS_EFFECTIVE) as i32;
    let num = state.mod_name_is_active(xkb::MOD_NAME_NUM, xkb::STATE_MODS_EFFECTIVE) as i32;
    let modifiers = shift | ctrl << 1 | alt << 2 | logo << 3 | caps << 4 | num << 5;

    // Send key event to all engines
    engines.for_each(move |engine| {
        engine.run_on_platform_thread(move |engine| {
            engine.with_plugin(move |plugin: &KeyEventPlugin| {
                plugin.key_action(KeyAction {
                    toolkit: "glfw".to_string(),
                    key_code: keycode,
                    scan_code: scancode as i32,
                    modifiers,
                    keymap: "linux".to_string(),
                    _type: match keystate {
                        KeyState::Released => KeyActionType::Keyup,
                        KeyState::Pressed => KeyActionType::Keydown,
                    },
                });
            });
        });
    });

    // Send text events
    if !content.is_empty()
        && keystate == KeyState::Pressed
        && content.chars().all(|x| !x.is_control())
    {
        let textinput = textinput.lock();
        if let Some(engine) = textinput.as_ref() {
            if let Some(engine) = engine.upgrade() {
                engine.run_on_platform_thread(move |engine| {
                    engine.with_plugin_mut(move |plugin: &mut TextInputPlugin| {
                        plugin.with_state(|state| {
                            state.add_characters(&content);
                        });
                        plugin.notify_changes();
                    });
                });
            }
        }
    }
}
