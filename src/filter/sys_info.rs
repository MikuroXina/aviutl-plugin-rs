use crate::{from_nullable_lpstr, from_win_str, Size};
use aviutl_plugin_sys::filter::SysInfo as RawSysInfo;
use std::{ffi::CStr, os::raw::c_char};
use windows::Win32::Graphics::Gdi::HFONT;

pub use aviutl_plugin_sys::filter::SysInfoFlag;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SysInfo {
    pub flags: SysInfoFlag,
    pub info: String,
    pub build_revision: u32,
    pub min_size: Size,
    pub max_size: Size,
    pub vram_size: Size,
    pub vram_bytes_per_pixel: usize,
    pub max_frames: usize,
    pub editing_file_name: Option<String>,
    pub project_file_name: Option<String>,
    pub exported_file_name: Option<String>,
    pub default_font: HFONT,
}

impl SysInfo {
    pub(crate) fn from_raw(raw: &RawSysInfo) -> Self {
        let vram_size = Size {
            width: raw.vram_w as u32,
            height: raw.vram_h as u32,
        };
        debug_assert_eq!(
            raw.vram_yc_size as usize % vram_size.area(),
            0,
            "vram_yc_size was not aligned"
        );
        Self {
            flags: raw.flag,
            info: from_win_str(unsafe {
                CStr::from_ptr(raw.info as *const _ as *const c_char).to_bytes()
            })
            .to_string(),
            build_revision: raw.build as u32,
            min_size: Size {
                width: raw.min_w as u32,
                height: raw.min_h as u32,
            },
            max_size: Size {
                width: raw.max_w as u32,
                height: raw.max_h as u32,
            },
            vram_size,
            vram_bytes_per_pixel: raw.vram_yc_size as usize / vram_size.area(),
            max_frames: raw.max_frame as usize,
            editing_file_name: unsafe { from_nullable_lpstr(raw.edit_name) },
            project_file_name: unsafe { from_nullable_lpstr(raw.project_name) },
            exported_file_name: unsafe { from_nullable_lpstr(raw.output_name) },
            default_font: HFONT(raw.font_handle),
        }
    }
}
