use aviutl_plugin_sys::filter::{WindowMessage as RawWindowMessage, MID_FILTER_BUTTON};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

use crate::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WindowMessage {
    Update,
    FileOpen,
    FileClose,
    Init,
    Exit,
    SaveStart,
    SaveEnd,
    Import,
    Export,
    ChangeActive,
    ChangeWindow,
    ChangeParam,
    ChangeEdit,
    Command {
        index: u32,
    },
    FileUpdate,
    MainMouseDown {
        coordinate: Point,
    },
    MainMouseUp {
        coordinate: Point,
    },
    MainMouseMove {
        coordinate: Point,
    },
    MainKeyDown {
        key_code: KeyCode,
    },
    MainKeyUp {
        key_code: KeyCode,
    },
    MainMoveSize {
        window: HWND,
    },
    MainMouseDoubleClick {
        coordinate: Point,
    },
    MainMouseRightDown {
        coordinate: Point,
    },
    MainMouseRightUp {
        coordinate: Point,
    },
    MainMouseWheel {
        amount: i16,
    },
    MainContextMenu {
        coordinate: Point,
    },
    System {
        original: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    },
}

impl WindowMessage {
    pub fn from(raw: RawWindowMessage, wparam: WPARAM, lparam: LPARAM) -> Self {
        fn extract_pos(lparam: LPARAM) -> Point {
            let bytes = lparam.0.to_le_bytes();
            let lower = i16::from_le_bytes([bytes[0], bytes[1]]);
            let higher = i16::from_le_bytes([bytes[2], bytes[3]]);
            Point {
                x: lower as i16 as i32,
                y: higher as i16 as i32,
            }
        }
        match raw {
            RawWindowMessage::Update => WindowMessage::Update,
            RawWindowMessage::FileOpen => WindowMessage::FileOpen,
            RawWindowMessage::FileClose => WindowMessage::FileClose,
            RawWindowMessage::Init => WindowMessage::Init,
            RawWindowMessage::Exit => WindowMessage::Exit,
            RawWindowMessage::SaveStart => WindowMessage::SaveStart,
            RawWindowMessage::SaveEnd => WindowMessage::SaveEnd,
            RawWindowMessage::Import => WindowMessage::Import,
            RawWindowMessage::Export => WindowMessage::Export,
            RawWindowMessage::ChangeActive => WindowMessage::ChangeActive,
            RawWindowMessage::ChangeWindow => WindowMessage::ChangeWindow,
            RawWindowMessage::ChangeParam => WindowMessage::ChangeParam,
            RawWindowMessage::ChangeEdit => WindowMessage::ChangeEdit,
            RawWindowMessage::Command => WindowMessage::Command {
                index: (wparam.0 - MID_FILTER_BUTTON) as u32,
            },
            RawWindowMessage::FileUpdate => WindowMessage::FileUpdate,
            RawWindowMessage::MainMouseDown => WindowMessage::MainMouseDown {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainMouseUp => WindowMessage::MainMouseUp {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainMouseMove => WindowMessage::MainMouseMove {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainKeyDown => WindowMessage::MainKeyDown {
                key_code: KeyCode(wparam.0 as u8),
            },
            RawWindowMessage::MainKeyUp => WindowMessage::MainKeyUp {
                key_code: KeyCode(wparam.0 as u8),
            },
            RawWindowMessage::MainMoveSize => WindowMessage::MainMoveSize {
                window: HWND(lparam.0),
            },
            RawWindowMessage::MainMouseDoubleClick => WindowMessage::MainMouseDoubleClick {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainMouseRightDown => WindowMessage::MainMouseRightDown {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainMouseRightUp => WindowMessage::MainMouseRightUp {
                coordinate: extract_pos(lparam),
            },
            RawWindowMessage::MainMouseWheel => WindowMessage::MainMouseWheel {
                amount: (wparam.0 >> 16) as i16,
            },
            RawWindowMessage::MainContextMenu => WindowMessage::MainContextMenu {
                coordinate: extract_pos(lparam),
            },
            _ => WindowMessage::System {
                original: raw as u32,
                wparam,
                lparam,
            },
        }
    }
}
