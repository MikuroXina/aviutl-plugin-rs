//! From input plugin header for AviUtl version 0.99k or later by ＫＥＮくん.

use bitflags::bitflags;
use std::os::raw::{c_int, c_void};
use windows_sys::{
    core::PSTR as LPSTR,
    Win32::{
        Foundation::{BOOL, HINSTANCE, HWND},
        Graphics::Gdi::BITMAPINFOHEADER,
        Media::Audio::WAVEFORMATEX,
    },
};

/// Information of the input file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct InputInfo {
    /// Flags of the input file.
    pub flag: InputInfoFlag,
    /// The numerator of the frame rate.
    pub rate: c_int,
    /// The denominator of the frame rate.
    pub scale: c_int,
    /// The total frames of the video.
    pub n: c_int,
    /// The bitmap format. You can use RGB, YUY2 and the installed codecs. You need to complete initialization before your handler is called in next time. `u32::from_bytes([b'Y', b'C', b'4', b'8'])` will be YCbCr pixel format (`biBitCount` is 48) but this is unavailable on using YUY2 filter mode.
    pub format: *const BITMAPINFOHEADER,
    /// The size of `format`.
    pub format_size: c_int,
    /// The total samples of the audio data.
    pub audio_n: c_int,
    /// The audio format. You can use PCM and the installed codecs. You need to complete initialization before your handler is called in next time.
    pub audio_format: *const WAVEFORMATEX,
    /// The size of `audio_format`.
    pub audio_format_size: c_int,
    /// The video codec handler.
    pub handler: u32,
    /// Reserved.
    pub _reserve: [c_int; 7],
}

bitflags! {
    /// Flag of the input file.
    pub struct InputInfoFlag: c_int {
        /// Contains image/video data.
        const VIDEO = 1;
        /// Contains audio data.
        const AUDIO = 2;
        /// Your `func_read_video` will be called with random frame index. If not, it is called sequentially from the key frame.
        const VIDEO_RANDOM_ACCESS = 8;
    }
}

/// Your handle of the input file.
pub type InputHandle = *mut c_void;

bitflags! {
    /// Flag for the input plugin.
    pub struct InputPluginFlag: c_int {
        /// Supported some video format.
        const VIDEO = 1;
        /// Supported some audio format.
        const AUDIO = 2;
    }
}

/// Definition of the input plugin.
///
/// # Safety
///
/// You must use only for share the table to AviUtl.
#[repr(C)]
pub struct InputPlugin {
    /// Flags of the plugin.
    pub flag: InputPluginFlag,
    /// The plugin name.
    pub name: LPSTR,
    /// The file filter string.
    pub file_filter: LPSTR,
    /// The plugin information.
    pub information: LPSTR,
    /// Initialization handler of the input plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_init: unsafe extern "system" fn() -> BOOL,
    /// Exiting handler of the input plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_exit: unsafe extern "system" fn() -> BOOL,
    /// File open handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The file name.
    ///
    /// # Returns
    ///
    /// The handle of the input file if succeed, or null if failed.
    pub func_open: unsafe extern "system" fn(LPSTR) -> InputHandle,
    /// File close handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the input file.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_close: unsafe extern "system" fn(InputHandle) -> BOOL,
    /// File information handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the input file.
    /// 2. The pointer to write the input file information.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_info_get: unsafe extern "system" fn(InputHandle, *mut InputInfo) -> BOOL,
    /// Video read handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the input file.
    /// 2. The start frame index to read.
    /// 3. The pointer to write the read data. The buffer is allocated only the space for one frame.
    ///
    /// # Returns
    ///
    /// The size of the read data.
    pub func_read_video: unsafe extern "system" fn(InputHandle, c_int, *mut c_void) -> c_int,
    /// Audio read handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the input file.
    /// 2. The start frame index to read.
    /// 3. The number of samples to read.
    /// 4. The pointer to write the read data.
    ///
    /// # Returns
    ///
    /// The samples of the read data.
    pub func_read_audio: unsafe extern "system" fn(InputHandle, c_int, c_int, *mut c_void) -> c_int,
    /// Key frame check handler of the input plugin. If you provided null, all frames will treat as key frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the input file.
    /// 2. The frame index to check.
    ///
    /// # Returns
    ///
    /// Whether the frame is a key frame.
    pub func_is_keyframe: unsafe extern "system" fn(InputHandle, c_int) -> BOOL,
    /// Configuration dialog handler of the input plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the configuration window.
    /// 2. The handle of the DLL instance.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_config: unsafe extern "system" fn(HWND, HINSTANCE) -> BOOL,
    /// Reserved.
    pub _reserve: [c_int; 16],
}

unsafe impl Send for InputPlugin {}
unsafe impl Sync for InputPlugin {}
