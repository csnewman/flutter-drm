#![allow(dead_code)]

pub fn map_key(code: u32) -> i32 {
    if code >= GLFW_MAPPING.len() as u32 {
        GLFW_KEY_UNKNOWN
    } else {
        GLFW_MAPPING[code as usize]
    }
}

pub const GLFW_KEY_UNKNOWN: i32 = -1;

/* Printable keys */
pub const GLFW_KEY_SPACE: i32 = 32;
pub const GLFW_KEY_APOSTROPHE: i32 = 39; /* ' */
pub const GLFW_KEY_COMMA: i32 = 44; /* , */
pub const GLFW_KEY_MINUS: i32 = 45; /* - */
pub const GLFW_KEY_PERIOD: i32 = 46; /* . */
pub const GLFW_KEY_SLASH: i32 = 47; /* / */
pub const GLFW_KEY_0: i32 = 48;
pub const GLFW_KEY_1: i32 = 49;
pub const GLFW_KEY_2: i32 = 50;
pub const GLFW_KEY_3: i32 = 51;
pub const GLFW_KEY_4: i32 = 52;
pub const GLFW_KEY_5: i32 = 53;
pub const GLFW_KEY_6: i32 = 54;
pub const GLFW_KEY_7: i32 = 55;
pub const GLFW_KEY_8: i32 = 56;
pub const GLFW_KEY_9: i32 = 57;
pub const GLFW_KEY_SEMICOLON: i32 = 59; /* ; */
pub const GLFW_KEY_EQUAL: i32 = 61; /* = */
pub const GLFW_KEY_A: i32 = 65;
pub const GLFW_KEY_B: i32 = 66;
pub const GLFW_KEY_C: i32 = 67;
pub const GLFW_KEY_D: i32 = 68;
pub const GLFW_KEY_E: i32 = 69;
pub const GLFW_KEY_F: i32 = 70;
pub const GLFW_KEY_G: i32 = 71;
pub const GLFW_KEY_H: i32 = 72;
pub const GLFW_KEY_I: i32 = 73;
pub const GLFW_KEY_J: i32 = 74;
pub const GLFW_KEY_K: i32 = 75;
pub const GLFW_KEY_L: i32 = 76;
pub const GLFW_KEY_M: i32 = 77;
pub const GLFW_KEY_N: i32 = 78;
pub const GLFW_KEY_O: i32 = 79;
pub const GLFW_KEY_P: i32 = 80;
pub const GLFW_KEY_Q: i32 = 81;
pub const GLFW_KEY_R: i32 = 82;
pub const GLFW_KEY_S: i32 = 83;
pub const GLFW_KEY_T: i32 = 84;
pub const GLFW_KEY_U: i32 = 85;
pub const GLFW_KEY_V: i32 = 86;
pub const GLFW_KEY_W: i32 = 87;
pub const GLFW_KEY_X: i32 = 88;
pub const GLFW_KEY_Y: i32 = 89;
pub const GLFW_KEY_Z: i32 = 90;
pub const GLFW_KEY_LEFT_BRACKET: i32 = 91; /* [ */
pub const GLFW_KEY_BACKSLASH: i32 = 92; /* \ */
pub const GLFW_KEY_RIGHT_BRACKET: i32 = 93; /* ] */
pub const GLFW_KEY_GRAVE_ACCENT: i32 = 96; /* ` */
pub const GLFW_KEY_WORLD_1: i32 = 161; /* non-US #1 */
pub const GLFW_KEY_WORLD_2: i32 = 162; /* non-US #2 */

/* Function keys */
pub const GLFW_KEY_ESCAPE: i32 = 256;
pub const GLFW_KEY_ENTER: i32 = 257;
pub const GLFW_KEY_TAB: i32 = 258;
pub const GLFW_KEY_BACKSPACE: i32 = 259;
pub const GLFW_KEY_INSERT: i32 = 260;
pub const GLFW_KEY_DELETE: i32 = 261;
pub const GLFW_KEY_RIGHT: i32 = 262;
pub const GLFW_KEY_LEFT: i32 = 263;
pub const GLFW_KEY_DOWN: i32 = 264;
pub const GLFW_KEY_UP: i32 = 265;
pub const GLFW_KEY_PAGE_UP: i32 = 266;
pub const GLFW_KEY_PAGE_DOWN: i32 = 267;
pub const GLFW_KEY_HOME: i32 = 268;
pub const GLFW_KEY_END: i32 = 269;
pub const GLFW_KEY_CAPS_LOCK: i32 = 280;
pub const GLFW_KEY_SCROLL_LOCK: i32 = 281;
pub const GLFW_KEY_NUM_LOCK: i32 = 282;
pub const GLFW_KEY_PRINT_SCREEN: i32 = 283;
pub const GLFW_KEY_PAUSE: i32 = 284;
pub const GLFW_KEY_F1: i32 = 290;
pub const GLFW_KEY_F2: i32 = 291;
pub const GLFW_KEY_F3: i32 = 292;
pub const GLFW_KEY_F4: i32 = 293;
pub const GLFW_KEY_F5: i32 = 294;
pub const GLFW_KEY_F6: i32 = 295;
pub const GLFW_KEY_F7: i32 = 296;
pub const GLFW_KEY_F8: i32 = 297;
pub const GLFW_KEY_F9: i32 = 298;
pub const GLFW_KEY_F10: i32 = 299;
pub const GLFW_KEY_F11: i32 = 300;
pub const GLFW_KEY_F12: i32 = 301;
pub const GLFW_KEY_F13: i32 = 302;
pub const GLFW_KEY_F14: i32 = 303;
pub const GLFW_KEY_F15: i32 = 304;
pub const GLFW_KEY_F16: i32 = 305;
pub const GLFW_KEY_F17: i32 = 306;
pub const GLFW_KEY_F18: i32 = 307;
pub const GLFW_KEY_F19: i32 = 308;
pub const GLFW_KEY_F20: i32 = 309;
pub const GLFW_KEY_F21: i32 = 310;
pub const GLFW_KEY_F22: i32 = 311;
pub const GLFW_KEY_F23: i32 = 312;
pub const GLFW_KEY_F24: i32 = 313;
pub const GLFW_KEY_F25: i32 = 314;
pub const GLFW_KEY_KP_0: i32 = 320;
pub const GLFW_KEY_KP_1: i32 = 321;
pub const GLFW_KEY_KP_2: i32 = 322;
pub const GLFW_KEY_KP_3: i32 = 323;
pub const GLFW_KEY_KP_4: i32 = 324;
pub const GLFW_KEY_KP_5: i32 = 325;
pub const GLFW_KEY_KP_6: i32 = 326;
pub const GLFW_KEY_KP_7: i32 = 327;
pub const GLFW_KEY_KP_8: i32 = 328;
pub const GLFW_KEY_KP_9: i32 = 329;
pub const GLFW_KEY_KP_DECIMAL: i32 = 330;
pub const GLFW_KEY_KP_DIVIDE: i32 = 331;
pub const GLFW_KEY_KP_MULTIPLY: i32 = 332;
pub const GLFW_KEY_KP_SUBTRACT: i32 = 333;
pub const GLFW_KEY_KP_ADD: i32 = 334;
pub const GLFW_KEY_KP_ENTER: i32 = 335;
pub const GLFW_KEY_KP_EQUAL: i32 = 336;
pub const GLFW_KEY_LEFT_SHIFT: i32 = 340;
pub const GLFW_KEY_LEFT_CONTROL: i32 = 341;
pub const GLFW_KEY_LEFT_ALT: i32 = 342;
pub const GLFW_KEY_LEFT_SUPER: i32 = 343;
pub const GLFW_KEY_RIGHT_SHIFT: i32 = 344;
pub const GLFW_KEY_RIGHT_CONTROL: i32 = 345;
pub const GLFW_KEY_RIGHT_ALT: i32 = 346;
pub const GLFW_KEY_RIGHT_SUPER: i32 = 347;
pub const GLFW_KEY_MENU: i32 = 348;

pub const GLFW_KEY_LAST: i32 = GLFW_KEY_MENU;

pub const GLFW_MAPPING: [i32; 256] = [
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_ESCAPE,
    GLFW_KEY_1,
    GLFW_KEY_2,
    GLFW_KEY_3,
    GLFW_KEY_4,
    GLFW_KEY_5,
    GLFW_KEY_6,
    GLFW_KEY_7,
    GLFW_KEY_8,
    GLFW_KEY_9,
    GLFW_KEY_0,
    GLFW_KEY_MINUS,
    GLFW_KEY_EQUAL,
    GLFW_KEY_BACKSPACE,
    GLFW_KEY_TAB,
    GLFW_KEY_Q,
    GLFW_KEY_W,
    GLFW_KEY_E,
    GLFW_KEY_R,
    GLFW_KEY_T,
    GLFW_KEY_Y,
    GLFW_KEY_U,
    GLFW_KEY_I,
    GLFW_KEY_O,
    GLFW_KEY_P,
    GLFW_KEY_LEFT_BRACKET,
    GLFW_KEY_RIGHT_BRACKET,
    GLFW_KEY_ENTER,
    GLFW_KEY_LEFT_CONTROL,
    GLFW_KEY_A,
    GLFW_KEY_S,
    GLFW_KEY_D,
    GLFW_KEY_F,
    GLFW_KEY_G,
    GLFW_KEY_H,
    GLFW_KEY_J,
    GLFW_KEY_K,
    GLFW_KEY_L,
    GLFW_KEY_SEMICOLON,
    GLFW_KEY_APOSTROPHE,
    GLFW_KEY_GRAVE_ACCENT,
    GLFW_KEY_LEFT_SHIFT,
    GLFW_KEY_BACKSLASH,
    GLFW_KEY_Z,
    GLFW_KEY_X,
    GLFW_KEY_C,
    GLFW_KEY_V,
    GLFW_KEY_B,
    GLFW_KEY_N,
    GLFW_KEY_M,
    GLFW_KEY_COMMA,
    GLFW_KEY_PERIOD,
    GLFW_KEY_SLASH,
    GLFW_KEY_RIGHT_SHIFT,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_LEFT_ALT,
    GLFW_KEY_SPACE,
    GLFW_KEY_CAPS_LOCK,
    GLFW_KEY_F1,
    GLFW_KEY_F2,
    GLFW_KEY_F3,
    GLFW_KEY_F4,
    GLFW_KEY_F5,
    GLFW_KEY_F6,
    GLFW_KEY_F7,
    GLFW_KEY_F8,
    GLFW_KEY_F9,
    GLFW_KEY_F10,
    GLFW_KEY_NUM_LOCK,
    GLFW_KEY_SCROLL_LOCK,
    GLFW_KEY_KP_7,
    GLFW_KEY_KP_8,
    GLFW_KEY_KP_9,
    GLFW_KEY_KP_SUBTRACT,
    GLFW_KEY_KP_4,
    GLFW_KEY_KP_5,
    GLFW_KEY_KP_6,
    GLFW_KEY_KP_ADD,
    GLFW_KEY_KP_1,
    GLFW_KEY_KP_2,
    GLFW_KEY_KP_3,
    GLFW_KEY_KP_0,
    GLFW_KEY_KP_MULTIPLY,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_F11,
    GLFW_KEY_F12,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_KP_ENTER,
    GLFW_KEY_RIGHT_CONTROL,
    GLFW_KEY_KP_DIVIDE,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_RIGHT_ALT,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_HOME,
    GLFW_KEY_UP,
    GLFW_KEY_PAGE_UP,
    GLFW_KEY_LEFT,
    GLFW_KEY_RIGHT,
    GLFW_KEY_END,
    GLFW_KEY_DOWN,
    GLFW_KEY_PAGE_DOWN,
    GLFW_KEY_INSERT,
    GLFW_KEY_DELETE,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_KP_EQUAL,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_PAUSE,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_KP_DECIMAL,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_LEFT_SUPER,
    GLFW_KEY_RIGHT_SUPER,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_MENU,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_F13,
    GLFW_KEY_F14,
    GLFW_KEY_F15,
    GLFW_KEY_F16,
    GLFW_KEY_F17,
    GLFW_KEY_F18,
    GLFW_KEY_F19,
    GLFW_KEY_F20,
    GLFW_KEY_F21,
    GLFW_KEY_F22,
    GLFW_KEY_F23,
    GLFW_KEY_F24,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_PRINT_SCREEN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
    GLFW_KEY_UNKNOWN,
];
