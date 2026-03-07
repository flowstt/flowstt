//! Global hotkey listener for FlowSTT.
//!
//! Monitors physical key press/release events at the OS level and triggers
//! engine `start_recording()` / `stop_recording()` when the configured PTT
//! hotkey combination is held/released.
//!
//! This module is a pure app-level concern — vtx-engine knows nothing about
//! hotkeys or push-to-talk; it simply provides a manual recording API.
//!
//! ## Platform backends
//!
//! - **Windows**: Uses the Raw Input API (`RegisterRawInputDevices` with
//!   `RIDEV_INPUTSINK`) on a hidden message-only window.  This is the same
//!   approach the original flowstt-engine used and is known to work reliably.
//! - **Other platforms**: Falls back to `rdev::listen` (low-level keyboard hook).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use tracing::{debug, error, info};
use vtx_engine::{AudioEngine, KeyCode};

use flowstt_common::HotkeyCombination;

// ─── Windows Raw Input backend ──────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod win_raw_input {
    use super::*;
    use std::mem::size_of;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::{
        GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
        RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
        RegisterClassW, TranslateMessage, UnregisterClassW, HWND_MESSAGE, MSG, PM_REMOVE, WM_INPUT,
        WM_QUIT, WNDCLASSW, WS_OVERLAPPED,
    };

    /// Raw input keyboard flags
    const RI_KEY_BREAK: u16 = 1;
    const RI_KEY_E0: u16 = 2;

    /// Windows virtual-key codes
    mod vk {
        pub const MENU: u16 = 0x12;
        pub const CONTROL: u16 = 0x11;
        pub const SHIFT: u16 = 0x10;
        pub const LWIN: u16 = 0x5B;
        pub const RWIN: u16 = 0x5C;
        pub const CAPS_LOCK: u16 = 0x14;
        pub const F1: u16 = 0x70;
        pub const F2: u16 = 0x71;
        pub const F3: u16 = 0x72;
        pub const F4: u16 = 0x73;
        pub const F5: u16 = 0x74;
        pub const F6: u16 = 0x75;
        pub const F7: u16 = 0x76;
        pub const F8: u16 = 0x77;
        pub const F9: u16 = 0x78;
        pub const F10: u16 = 0x79;
        pub const F11: u16 = 0x7A;
        pub const F12: u16 = 0x7B;
        pub const F13: u16 = 0x7C;
        pub const F14: u16 = 0x7D;
        pub const F15: u16 = 0x7E;
        pub const F16: u16 = 0x7F;
        pub const F17: u16 = 0x80;
        pub const F18: u16 = 0x81;
        pub const F19: u16 = 0x82;
        pub const F20: u16 = 0x83;
        pub const F21: u16 = 0x84;
        pub const F22: u16 = 0x85;
        pub const F23: u16 = 0x86;
        pub const F24: u16 = 0x87;
        pub const A: u16 = 0x41;
        pub const B: u16 = 0x42;
        pub const C: u16 = 0x43;
        pub const D: u16 = 0x44;
        pub const E: u16 = 0x45;
        pub const F: u16 = 0x46;
        pub const G: u16 = 0x47;
        pub const H: u16 = 0x48;
        pub const I: u16 = 0x49;
        pub const J: u16 = 0x4A;
        pub const K: u16 = 0x4B;
        pub const L: u16 = 0x4C;
        pub const M: u16 = 0x4D;
        pub const N: u16 = 0x4E;
        pub const O: u16 = 0x4F;
        pub const P: u16 = 0x50;
        pub const Q: u16 = 0x51;
        pub const R: u16 = 0x52;
        pub const S: u16 = 0x53;
        pub const T: u16 = 0x54;
        pub const U: u16 = 0x55;
        pub const V: u16 = 0x56;
        pub const W: u16 = 0x57;
        pub const X: u16 = 0x58;
        pub const Y: u16 = 0x59;
        pub const Z: u16 = 0x5A;
        pub const DIGIT_0: u16 = 0x30;
        pub const DIGIT_1: u16 = 0x31;
        pub const DIGIT_2: u16 = 0x32;
        pub const DIGIT_3: u16 = 0x33;
        pub const DIGIT_4: u16 = 0x34;
        pub const DIGIT_5: u16 = 0x35;
        pub const DIGIT_6: u16 = 0x36;
        pub const DIGIT_7: u16 = 0x37;
        pub const DIGIT_8: u16 = 0x38;
        pub const DIGIT_9: u16 = 0x39;
        pub const UP: u16 = 0x26;
        pub const DOWN: u16 = 0x28;
        pub const LEFT: u16 = 0x25;
        pub const RIGHT: u16 = 0x27;
        pub const HOME: u16 = 0x24;
        pub const END: u16 = 0x23;
        pub const PRIOR: u16 = 0x21;
        pub const NEXT: u16 = 0x22;
        pub const INSERT: u16 = 0x2D;
        pub const DELETE: u16 = 0x2E;
        pub const ESCAPE: u16 = 0x1B;
        pub const TAB: u16 = 0x09;
        pub const SPACE: u16 = 0x20;
        pub const RETURN: u16 = 0x0D;
        pub const BACK: u16 = 0x08;
        pub const SNAPSHOT: u16 = 0x2C;
        pub const SCROLL: u16 = 0x91;
        pub const PAUSE: u16 = 0x13;
        pub const OEM_MINUS: u16 = 0xBD;
        pub const OEM_PLUS: u16 = 0xBB;
        pub const OEM_4: u16 = 0xDB;
        pub const OEM_6: u16 = 0xDD;
        pub const OEM_5: u16 = 0xDC;
        pub const OEM_1: u16 = 0xBA;
        pub const OEM_7: u16 = 0xDE;
        pub const OEM_3: u16 = 0xC0;
        pub const OEM_COMMA: u16 = 0xBC;
        pub const OEM_PERIOD: u16 = 0xBE;
        pub const OEM_2: u16 = 0xBF;
        pub const NUMPAD0: u16 = 0x60;
        pub const NUMPAD1: u16 = 0x61;
        pub const NUMPAD2: u16 = 0x62;
        pub const NUMPAD3: u16 = 0x63;
        pub const NUMPAD4: u16 = 0x64;
        pub const NUMPAD5: u16 = 0x65;
        pub const NUMPAD6: u16 = 0x66;
        pub const NUMPAD7: u16 = 0x67;
        pub const NUMPAD8: u16 = 0x68;
        pub const NUMPAD9: u16 = 0x69;
        pub const MULTIPLY: u16 = 0x6A;
        pub const ADD: u16 = 0x6B;
        pub const SUBTRACT: u16 = 0x6D;
        pub const DECIMAL: u16 = 0x6E;
        pub const DIVIDE: u16 = 0x6F;
        pub const NUMLOCK: u16 = 0x90;
    }

    /// Convert a Raw Input VK code + E0 flag + MakeCode to a KeyCode.
    ///
    /// Shift keys need special handling: Raw Input does NOT set the E0 flag to
    /// distinguish Left/Right Shift.  Instead they have distinct scan codes:
    /// Left Shift = 0x2A, Right Shift = 0x36.
    fn raw_input_to_keycode(vk_code: u16, is_e0: bool, make_code: u16) -> Option<KeyCode> {
        match (vk_code, is_e0, make_code) {
            (vk::MENU, true, _) => Some(KeyCode::RightAlt),
            (vk::MENU, false, _) => Some(KeyCode::LeftAlt),
            (vk::CONTROL, true, _) => Some(KeyCode::RightControl),
            (vk::CONTROL, false, _) => Some(KeyCode::LeftControl),
            (vk::SHIFT, _, 0x36) => Some(KeyCode::RightShift),
            (vk::SHIFT, _, _) => Some(KeyCode::LeftShift),
            (vk::CAPS_LOCK, _, _) => Some(KeyCode::CapsLock),
            (vk::LWIN, _, _) => Some(KeyCode::LeftMeta),
            (vk::RWIN, _, _) => Some(KeyCode::RightMeta),
            (vk::F1, _, _) => Some(KeyCode::F1),
            (vk::F2, _, _) => Some(KeyCode::F2),
            (vk::F3, _, _) => Some(KeyCode::F3),
            (vk::F4, _, _) => Some(KeyCode::F4),
            (vk::F5, _, _) => Some(KeyCode::F5),
            (vk::F6, _, _) => Some(KeyCode::F6),
            (vk::F7, _, _) => Some(KeyCode::F7),
            (vk::F8, _, _) => Some(KeyCode::F8),
            (vk::F9, _, _) => Some(KeyCode::F9),
            (vk::F10, _, _) => Some(KeyCode::F10),
            (vk::F11, _, _) => Some(KeyCode::F11),
            (vk::F12, _, _) => Some(KeyCode::F12),
            (vk::F13, _, _) => Some(KeyCode::F13),
            (vk::F14, _, _) => Some(KeyCode::F14),
            (vk::F15, _, _) => Some(KeyCode::F15),
            (vk::F16, _, _) => Some(KeyCode::F16),
            (vk::F17, _, _) => Some(KeyCode::F17),
            (vk::F18, _, _) => Some(KeyCode::F18),
            (vk::F19, _, _) => Some(KeyCode::F19),
            (vk::F20, _, _) => Some(KeyCode::F20),
            (vk::F21, _, _) => Some(KeyCode::F21),
            (vk::F22, _, _) => Some(KeyCode::F22),
            (vk::F23, _, _) => Some(KeyCode::F23),
            (vk::F24, _, _) => Some(KeyCode::F24),
            (vk::A, _, _) => Some(KeyCode::KeyA),
            (vk::B, _, _) => Some(KeyCode::KeyB),
            (vk::C, _, _) => Some(KeyCode::KeyC),
            (vk::D, _, _) => Some(KeyCode::KeyD),
            (vk::E, _, _) => Some(KeyCode::KeyE),
            (vk::F, _, _) => Some(KeyCode::KeyF),
            (vk::G, _, _) => Some(KeyCode::KeyG),
            (vk::H, _, _) => Some(KeyCode::KeyH),
            (vk::I, _, _) => Some(KeyCode::KeyI),
            (vk::J, _, _) => Some(KeyCode::KeyJ),
            (vk::K, _, _) => Some(KeyCode::KeyK),
            (vk::L, _, _) => Some(KeyCode::KeyL),
            (vk::M, _, _) => Some(KeyCode::KeyM),
            (vk::N, _, _) => Some(KeyCode::KeyN),
            (vk::O, _, _) => Some(KeyCode::KeyO),
            (vk::P, _, _) => Some(KeyCode::KeyP),
            (vk::Q, _, _) => Some(KeyCode::KeyQ),
            (vk::R, _, _) => Some(KeyCode::KeyR),
            (vk::S, _, _) => Some(KeyCode::KeyS),
            (vk::T, _, _) => Some(KeyCode::KeyT),
            (vk::U, _, _) => Some(KeyCode::KeyU),
            (vk::V, _, _) => Some(KeyCode::KeyV),
            (vk::W, _, _) => Some(KeyCode::KeyW),
            (vk::X, _, _) => Some(KeyCode::KeyX),
            (vk::Y, _, _) => Some(KeyCode::KeyY),
            (vk::Z, _, _) => Some(KeyCode::KeyZ),
            (vk::DIGIT_0, _, _) => Some(KeyCode::Digit0),
            (vk::DIGIT_1, _, _) => Some(KeyCode::Digit1),
            (vk::DIGIT_2, _, _) => Some(KeyCode::Digit2),
            (vk::DIGIT_3, _, _) => Some(KeyCode::Digit3),
            (vk::DIGIT_4, _, _) => Some(KeyCode::Digit4),
            (vk::DIGIT_5, _, _) => Some(KeyCode::Digit5),
            (vk::DIGIT_6, _, _) => Some(KeyCode::Digit6),
            (vk::DIGIT_7, _, _) => Some(KeyCode::Digit7),
            (vk::DIGIT_8, _, _) => Some(KeyCode::Digit8),
            (vk::DIGIT_9, _, _) => Some(KeyCode::Digit9),
            (vk::UP, _, _) => Some(KeyCode::ArrowUp),
            (vk::DOWN, _, _) => Some(KeyCode::ArrowDown),
            (vk::LEFT, _, _) => Some(KeyCode::ArrowLeft),
            (vk::RIGHT, _, _) => Some(KeyCode::ArrowRight),
            (vk::HOME, _, _) => Some(KeyCode::Home),
            (vk::END, _, _) => Some(KeyCode::End),
            (vk::PRIOR, _, _) => Some(KeyCode::PageUp),
            (vk::NEXT, _, _) => Some(KeyCode::PageDown),
            (vk::INSERT, _, _) => Some(KeyCode::Insert),
            (vk::DELETE, _, _) => Some(KeyCode::Delete),
            (vk::ESCAPE, _, _) => Some(KeyCode::Escape),
            (vk::TAB, _, _) => Some(KeyCode::Tab),
            (vk::SPACE, _, _) => Some(KeyCode::Space),
            (vk::RETURN, _, _) => Some(KeyCode::Enter),
            (vk::BACK, _, _) => Some(KeyCode::Backspace),
            (vk::SNAPSHOT, _, _) => Some(KeyCode::PrintScreen),
            (vk::SCROLL, _, _) => Some(KeyCode::ScrollLock),
            (vk::PAUSE, _, _) => Some(KeyCode::Pause),
            (vk::OEM_MINUS, _, _) => Some(KeyCode::Minus),
            (vk::OEM_PLUS, _, _) => Some(KeyCode::Equal),
            (vk::OEM_4, _, _) => Some(KeyCode::BracketLeft),
            (vk::OEM_6, _, _) => Some(KeyCode::BracketRight),
            (vk::OEM_5, _, _) => Some(KeyCode::Backslash),
            (vk::OEM_1, _, _) => Some(KeyCode::Semicolon),
            (vk::OEM_7, _, _) => Some(KeyCode::Quote),
            (vk::OEM_3, _, _) => Some(KeyCode::Backquote),
            (vk::OEM_COMMA, _, _) => Some(KeyCode::Comma),
            (vk::OEM_PERIOD, _, _) => Some(KeyCode::Period),
            (vk::OEM_2, _, _) => Some(KeyCode::Slash),
            (vk::NUMPAD0, _, _) => Some(KeyCode::Numpad0),
            (vk::NUMPAD1, _, _) => Some(KeyCode::Numpad1),
            (vk::NUMPAD2, _, _) => Some(KeyCode::Numpad2),
            (vk::NUMPAD3, _, _) => Some(KeyCode::Numpad3),
            (vk::NUMPAD4, _, _) => Some(KeyCode::Numpad4),
            (vk::NUMPAD5, _, _) => Some(KeyCode::Numpad5),
            (vk::NUMPAD6, _, _) => Some(KeyCode::Numpad6),
            (vk::NUMPAD7, _, _) => Some(KeyCode::Numpad7),
            (vk::NUMPAD8, _, _) => Some(KeyCode::Numpad8),
            (vk::NUMPAD9, _, _) => Some(KeyCode::Numpad9),
            (vk::MULTIPLY, _, _) => Some(KeyCode::NumpadMultiply),
            (vk::ADD, _, _) => Some(KeyCode::NumpadAdd),
            (vk::SUBTRACT, _, _) => Some(KeyCode::NumpadSubtract),
            (vk::DECIMAL, _, _) => Some(KeyCode::NumpadDecimal),
            (vk::DIVIDE, _, _) => Some(KeyCode::NumpadDivide),
            (vk::NUMLOCK, _, _) => Some(KeyCode::NumLock),
            _ => None,
        }
    }

    /// Window procedure for handling WM_INPUT messages.
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_INPUT {
            return LRESULT(0);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    /// Process a raw keyboard input event, returning the keycode and is_key_up.
    unsafe fn process_raw_input(lparam: LPARAM) -> Option<(KeyCode, bool)> {
        let hrawinput = HRAWINPUT(lparam.0 as _);
        let mut size: u32 = 0;
        GetRawInputData(
            hrawinput,
            RID_INPUT,
            None,
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        let bytes_copied = GetRawInputData(
            hrawinput,
            RID_INPUT,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if bytes_copied != size {
            return None;
        }

        let raw_input = &*(buffer.as_ptr() as *const RAWINPUT);
        if raw_input.header.dwType != RIM_TYPEKEYBOARD.0 {
            return None;
        }

        let keyboard = &raw_input.data.keyboard;
        let vk_code = keyboard.VKey;
        let make_code = keyboard.MakeCode;
        let flags = keyboard.Flags;
        let is_key_up = (flags & RI_KEY_BREAK) != 0;
        let is_e0 = (flags & RI_KEY_E0) != 0;

        raw_input_to_keycode(vk_code, is_e0, make_code).map(|kc| (kc, is_key_up))
    }

    /// Run the Raw Input message loop.  Blocks until `stop_flag` is set or
    /// WM_QUIT is received.
    pub(super) fn run_message_loop(
        stop_flag: Arc<AtomicBool>,
        engine: Arc<AudioEngine>,
        initial_combos: Vec<HotkeyCombination>,
        combos_shared: Arc<Mutex<Vec<HotkeyCombination>>>,
        generation: Arc<AtomicU64>,
        tid_sender: std::sync::mpsc::Sender<u32>,
    ) {
        unsafe {
            let thread_id = GetCurrentThreadId();
            let _ = tid_sender.send(thread_id);

            let class_name = windows::core::w!("FlowSTT_HotkeyClass");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                lpszClassName: class_name,
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                let err = windows::Win32::Foundation::GetLastError();
                if err != windows::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS {
                    error!("[Hotkey] Failed to register window class ({:?})", err);
                    return;
                }
            }

            let hwnd = match CreateWindowExW(
                Default::default(),
                class_name,
                windows::core::w!("FlowSTT Hotkey"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            ) {
                Ok(h) => h,
                Err(e) => {
                    error!("[Hotkey] Failed to create message window: {}", e);
                    return;
                }
            };

            let rid = RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            };

            if let Err(e) = RegisterRawInputDevices(&[rid], size_of::<RAWINPUTDEVICE>() as u32) {
                error!("[Hotkey] Failed to register raw input device: {}", e);
                let _ = DestroyWindow(hwnd);
                return;
            }

            info!("[Hotkey] Raw input registered, message loop ready");

            let mut tracker = HotkeyTracker::new(&initial_combos);
            let mut last_gen: u64 = 0;
            let mut msg = MSG::default();

            while !stop_flag.load(Ordering::SeqCst) {
                // Check for combo updates
                let current_gen = generation.load(Ordering::Relaxed);
                if current_gen != last_gen {
                    if let Ok(new_combos) = combos_shared.try_lock() {
                        tracker.update_combos(&new_combos);
                        last_gen = current_gen;
                        debug!("[Hotkey] Combos updated ({} combo(s))", new_combos.len());
                    }
                }

                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        debug!("[Hotkey] Received WM_QUIT");
                        stop_flag.store(true, Ordering::SeqCst);
                        break;
                    }

                    if msg.message == WM_INPUT {
                        if let Some((keycode, is_key_up)) = process_raw_input(msg.lParam) {
                            let transition = if is_key_up {
                                tracker.key_up(keycode)
                            } else {
                                tracker.key_down(keycode)
                            };

                            match transition {
                                Transition::Pressed => {
                                    debug!("[Hotkey] PTT hotkey pressed — starting recording");
                                    engine.start_recording();
                                }
                                Transition::Released => {
                                    debug!("[Hotkey] PTT hotkey released — stopping recording");
                                    engine.stop_recording();
                                }
                                Transition::None => {}
                            }
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                std::thread::sleep(std::time::Duration::from_millis(5));
            }

            let _ = DestroyWindow(hwnd);
            let _ = UnregisterClassW(class_name, None);
            debug!("[Hotkey] Message loop cleaned up");
        }
    }
}

// ─── rdev fallback for non-Windows platforms ────────────────────────────────

#[cfg(not(target_os = "windows"))]
mod rdev_backend {
    use super::*;
    use rdev::{self, EventType, Key as RdevKey};

    fn rdev_key_to_keycode(key: &RdevKey) -> Option<KeyCode> {
        match key {
            RdevKey::ShiftLeft => Some(KeyCode::LeftShift),
            RdevKey::ShiftRight => Some(KeyCode::RightShift),
            RdevKey::ControlLeft => Some(KeyCode::LeftControl),
            RdevKey::ControlRight => Some(KeyCode::RightControl),
            RdevKey::Alt => Some(KeyCode::LeftAlt),
            RdevKey::AltGr => Some(KeyCode::RightAlt),
            RdevKey::MetaLeft => Some(KeyCode::LeftMeta),
            RdevKey::MetaRight => Some(KeyCode::RightMeta),
            RdevKey::CapsLock => Some(KeyCode::CapsLock),
            RdevKey::F1 => Some(KeyCode::F1),
            RdevKey::F2 => Some(KeyCode::F2),
            RdevKey::F3 => Some(KeyCode::F3),
            RdevKey::F4 => Some(KeyCode::F4),
            RdevKey::F5 => Some(KeyCode::F5),
            RdevKey::F6 => Some(KeyCode::F6),
            RdevKey::F7 => Some(KeyCode::F7),
            RdevKey::F8 => Some(KeyCode::F8),
            RdevKey::F9 => Some(KeyCode::F9),
            RdevKey::F10 => Some(KeyCode::F10),
            RdevKey::F11 => Some(KeyCode::F11),
            RdevKey::F12 => Some(KeyCode::F12),
            #[cfg(target_os = "linux")]
            RdevKey::Unknown(code) => match *code {
                183 => Some(KeyCode::F13),
                184 => Some(KeyCode::F14),
                185 => Some(KeyCode::F15),
                186 => Some(KeyCode::F16),
                187 => Some(KeyCode::F17),
                188 => Some(KeyCode::F18),
                189 => Some(KeyCode::F19),
                190 => Some(KeyCode::F20),
                191 => Some(KeyCode::F21),
                192 => Some(KeyCode::F22),
                193 => Some(KeyCode::F23),
                194 => Some(KeyCode::F24),
                _ => None,
            },
            RdevKey::Escape => Some(KeyCode::Escape),
            RdevKey::Tab => Some(KeyCode::Tab),
            RdevKey::Space => Some(KeyCode::Space),
            RdevKey::Return => Some(KeyCode::Enter),
            RdevKey::Backspace => Some(KeyCode::Backspace),
            RdevKey::Insert => Some(KeyCode::Insert),
            RdevKey::Delete => Some(KeyCode::Delete),
            RdevKey::Home => Some(KeyCode::Home),
            RdevKey::End => Some(KeyCode::End),
            RdevKey::PageUp => Some(KeyCode::PageUp),
            RdevKey::PageDown => Some(KeyCode::PageDown),
            RdevKey::UpArrow => Some(KeyCode::ArrowUp),
            RdevKey::DownArrow => Some(KeyCode::ArrowDown),
            RdevKey::LeftArrow => Some(KeyCode::ArrowLeft),
            RdevKey::RightArrow => Some(KeyCode::ArrowRight),
            RdevKey::PrintScreen => Some(KeyCode::PrintScreen),
            RdevKey::ScrollLock => Some(KeyCode::ScrollLock),
            RdevKey::Pause => Some(KeyCode::Pause),
            RdevKey::KeyA => Some(KeyCode::KeyA),
            RdevKey::KeyB => Some(KeyCode::KeyB),
            RdevKey::KeyC => Some(KeyCode::KeyC),
            RdevKey::KeyD => Some(KeyCode::KeyD),
            RdevKey::KeyE => Some(KeyCode::KeyE),
            RdevKey::KeyF => Some(KeyCode::KeyF),
            RdevKey::KeyG => Some(KeyCode::KeyG),
            RdevKey::KeyH => Some(KeyCode::KeyH),
            RdevKey::KeyI => Some(KeyCode::KeyI),
            RdevKey::KeyJ => Some(KeyCode::KeyJ),
            RdevKey::KeyK => Some(KeyCode::KeyK),
            RdevKey::KeyL => Some(KeyCode::KeyL),
            RdevKey::KeyM => Some(KeyCode::KeyM),
            RdevKey::KeyN => Some(KeyCode::KeyN),
            RdevKey::KeyO => Some(KeyCode::KeyO),
            RdevKey::KeyP => Some(KeyCode::KeyP),
            RdevKey::KeyQ => Some(KeyCode::KeyQ),
            RdevKey::KeyR => Some(KeyCode::KeyR),
            RdevKey::KeyS => Some(KeyCode::KeyS),
            RdevKey::KeyT => Some(KeyCode::KeyT),
            RdevKey::KeyU => Some(KeyCode::KeyU),
            RdevKey::KeyV => Some(KeyCode::KeyV),
            RdevKey::KeyW => Some(KeyCode::KeyW),
            RdevKey::KeyX => Some(KeyCode::KeyX),
            RdevKey::KeyY => Some(KeyCode::KeyY),
            RdevKey::KeyZ => Some(KeyCode::KeyZ),
            RdevKey::Num0 => Some(KeyCode::Digit0),
            RdevKey::Num1 => Some(KeyCode::Digit1),
            RdevKey::Num2 => Some(KeyCode::Digit2),
            RdevKey::Num3 => Some(KeyCode::Digit3),
            RdevKey::Num4 => Some(KeyCode::Digit4),
            RdevKey::Num5 => Some(KeyCode::Digit5),
            RdevKey::Num6 => Some(KeyCode::Digit6),
            RdevKey::Num7 => Some(KeyCode::Digit7),
            RdevKey::Num8 => Some(KeyCode::Digit8),
            RdevKey::Num9 => Some(KeyCode::Digit9),
            _ => None,
        }
    }

    pub(super) fn run_rdev_loop(
        stop: Arc<AtomicBool>,
        engine: Arc<AudioEngine>,
        initial_combos: Vec<HotkeyCombination>,
        combos_shared: Arc<Mutex<Vec<HotkeyCombination>>>,
        generation: Arc<AtomicU64>,
    ) {
        let mut tracker = HotkeyTracker::new(&initial_combos);
        let mut last_gen: u64 = 0;

        let callback = move |event: rdev::Event| {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            let current_gen = generation.load(Ordering::Relaxed);
            if current_gen != last_gen {
                if let Ok(new_combos) = combos_shared.try_lock() {
                    tracker.update_combos(&new_combos);
                    last_gen = current_gen;
                    debug!("[Hotkey] Combos updated ({} combo(s))", new_combos.len());
                }
            }
            let (key, is_press) = match event.event_type {
                EventType::KeyPress(k) => (k, true),
                EventType::KeyRelease(k) => (k, false),
                _ => return,
            };
            let keycode = match rdev_key_to_keycode(&key) {
                Some(kc) => kc,
                None => return,
            };
            let transition = if is_press {
                tracker.key_down(keycode)
            } else {
                tracker.key_up(keycode)
            };
            match transition {
                Transition::Pressed => {
                    debug!("[Hotkey] PTT hotkey pressed — starting recording");
                    engine.start_recording();
                }
                Transition::Released => {
                    debug!("[Hotkey] PTT hotkey released — stopping recording");
                    engine.stop_recording();
                }
                Transition::None => {}
            }
        };
        if let Err(e) = rdev::listen(callback) {
            error!("[Hotkey] Global key listener failed: {:?}", e);
        }
    }
}

// ─── Hotkey state tracker ───────────────────────────────────────────────────

/// Tracks which keys are currently held and whether the configured hotkey
/// combination is satisfied.
struct HotkeyTracker {
    held_keys: HashSet<KeyCode>,
    combos: Vec<HashSet<KeyCode>>,
    was_active: bool,
}

impl HotkeyTracker {
    fn new(hotkey_combos: &[HotkeyCombination]) -> Self {
        let combos = hotkey_combos
            .iter()
            .map(|h| h.keys.iter().cloned().collect::<HashSet<KeyCode>>())
            .collect();
        Self {
            held_keys: HashSet::new(),
            combos,
            was_active: false,
        }
    }

    fn key_down(&mut self, key: KeyCode) -> Transition {
        self.held_keys.insert(key);
        self.check_transition()
    }

    fn key_up(&mut self, key: KeyCode) -> Transition {
        self.held_keys.remove(&key);
        self.check_transition()
    }

    fn update_combos(&mut self, hotkey_combos: &[HotkeyCombination]) {
        self.combos = hotkey_combos
            .iter()
            .map(|h| h.keys.iter().cloned().collect::<HashSet<KeyCode>>())
            .collect();
    }

    fn is_active(&self) -> bool {
        self.combos
            .iter()
            .any(|combo| !combo.is_empty() && combo.is_subset(&self.held_keys))
    }

    fn check_transition(&mut self) -> Transition {
        let now_active = self.is_active();
        let transition = match (self.was_active, now_active) {
            (false, true) => Transition::Pressed,
            (true, false) => Transition::Released,
            _ => Transition::None,
        };
        self.was_active = now_active;
        transition
    }
}

#[derive(Debug, PartialEq)]
enum Transition {
    None,
    Pressed,
    Released,
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Handle to the running hotkey listener. Dropping this stops the listener.
pub struct HotkeyListener {
    stop_flag: Arc<AtomicBool>,
    combos: Arc<Mutex<Vec<HotkeyCombination>>>,
    generation: Arc<AtomicU64>,
    /// Windows: thread ID for posting WM_QUIT on shutdown.
    #[cfg(target_os = "windows")]
    thread_id: Option<u32>,
}

impl HotkeyListener {
    /// Start a global hotkey listener on a background thread.
    pub fn start(engine: Arc<AudioEngine>, hotkey_combos: Vec<HotkeyCombination>) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let combos = Arc::new(Mutex::new(hotkey_combos.clone()));
        let generation = Arc::new(AtomicU64::new(0));

        #[cfg(target_os = "windows")]
        let thread_id = {
            let sf = stop_flag.clone();
            let cs = combos.clone();
            let gs = generation.clone();
            let ec = engine.clone();
            let ic = hotkey_combos.clone();
            let (tid_tx, tid_rx) = std::sync::mpsc::channel();

            thread::spawn(move || {
                info!(
                    "[Hotkey] Listener starting with {} combo(s) (Windows Raw Input)",
                    ic.len()
                );
                win_raw_input::run_message_loop(sf, ec, ic, cs, gs, tid_tx);
                info!("[Hotkey] Listener thread exiting");
            });

            let tid = tid_rx.recv_timeout(std::time::Duration::from_secs(5)).ok();
            if tid.is_some() {
                info!("[Hotkey] Message loop thread started (tid={:?})", tid);
            } else {
                error!("[Hotkey] Failed to get message loop thread ID");
            }
            tid
        };

        #[cfg(not(target_os = "windows"))]
        {
            let sf = stop_flag.clone();
            let cs = combos.clone();
            let gs = generation.clone();
            let ec = engine.clone();
            let ic = hotkey_combos.clone();

            thread::spawn(move || {
                info!(
                    "[Hotkey] Listener starting with {} combo(s) (rdev)",
                    ic.len()
                );
                rdev_backend::run_rdev_loop(sf, ec, ic, cs, gs);
                info!("[Hotkey] Listener thread exiting");
            });
        }

        Self {
            stop_flag,
            combos,
            generation,
            #[cfg(target_os = "windows")]
            thread_id,
        }
    }

    /// Signal the listener thread to stop.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        #[cfg(target_os = "windows")]
        if let Some(tid) = self.thread_id {
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    tid,
                    windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
                    windows::Win32::Foundation::WPARAM(0),
                    windows::Win32::Foundation::LPARAM(0),
                );
            }
        }

        info!("[Hotkey] Listener stop requested");
    }

    /// Update the hotkey combinations at runtime.
    pub fn update_combos(&self, hotkey_combos: Vec<HotkeyCombination>) {
        if let Ok(mut combos) = self.combos.lock() {
            *combos = hotkey_combos;
        }
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl Drop for HotkeyListener {
    fn drop(&mut self) {
        self.stop();
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn combo(keys: &[KeyCode]) -> HotkeyCombination {
        HotkeyCombination::new(keys.to_vec())
    }

    #[test]
    fn single_key_combo() {
        let mut tracker = HotkeyTracker::new(&[combo(&[KeyCode::F13])]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::Released);
    }

    #[test]
    fn two_key_combo_press_order() {
        let mut tracker =
            HotkeyTracker::new(&[combo(&[KeyCode::RightControl, KeyCode::RightShift])]);
        assert_eq!(tracker.key_down(KeyCode::RightControl), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::RightShift), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::RightControl), Transition::Released);
        assert_eq!(tracker.key_up(KeyCode::RightShift), Transition::None);
    }

    #[test]
    fn two_key_combo_reverse_order() {
        let mut tracker =
            HotkeyTracker::new(&[combo(&[KeyCode::RightControl, KeyCode::RightShift])]);
        assert_eq!(tracker.key_down(KeyCode::RightShift), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::RightControl), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::RightShift), Transition::Released);
    }

    #[test]
    fn multiple_combos_either_triggers() {
        let mut tracker = HotkeyTracker::new(&[
            combo(&[KeyCode::F13]),
            combo(&[KeyCode::RightControl, KeyCode::RightShift]),
        ]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::Released);
        assert_eq!(tracker.key_down(KeyCode::RightControl), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::RightShift), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::RightShift), Transition::Released);
        assert_eq!(tracker.key_up(KeyCode::RightControl), Transition::None);
    }

    #[test]
    fn empty_combo_never_triggers() {
        let mut tracker = HotkeyTracker::new(&[combo(&[])]);
        assert_eq!(tracker.key_down(KeyCode::KeyA), Transition::None);
        assert_eq!(tracker.key_up(KeyCode::KeyA), Transition::None);
    }

    #[test]
    fn no_combos_never_triggers() {
        let mut tracker = HotkeyTracker::new(&[]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::None);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::None);
    }

    #[test]
    fn superset_keys_still_triggers() {
        let mut tracker =
            HotkeyTracker::new(&[combo(&[KeyCode::RightControl, KeyCode::RightShift])]);
        assert_eq!(tracker.key_down(KeyCode::KeyA), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::RightControl), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::RightShift), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::RightShift), Transition::Released);
    }

    #[test]
    fn update_combos_takes_effect() {
        let mut tracker = HotkeyTracker::new(&[combo(&[KeyCode::F13])]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::Released);
        tracker.update_combos(&[combo(&[KeyCode::F14])]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::None);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::F14), Transition::Pressed);
        assert_eq!(tracker.key_up(KeyCode::F14), Transition::Released);
    }

    #[test]
    fn held_keys_do_not_double_fire() {
        let mut tracker = HotkeyTracker::new(&[combo(&[KeyCode::F13])]);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::Pressed);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::None);
        assert_eq!(tracker.key_down(KeyCode::F13), Transition::None);
        assert_eq!(tracker.key_up(KeyCode::F13), Transition::Released);
    }
}
