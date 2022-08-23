//! From output plugin header for AviUtl version 0.99g4 or later by ＫＥＮくん.

use bitflags::bitflags;
use std::os::raw::{c_int, c_void};
use windows_sys::{
    core::PSTR as LPSTR,
    Win32::Foundation::{BOOL, HINSTANCE, HWND},
};

bitflags! {
    /// Flag for the output information.
    pub struct OutputInfoFlag: c_int {
        /// Contains video data.
        const VIDEO = 1;
        /// Contains audio data.
        const AUDIO = 2;
        /// Batch outputting.
        const BATCH = 4;
    }
}

bitflags! {
    /// Flag for the frame.
    pub struct FrameFlag: c_int {
        /// You should treat as a key frame.
        const KEY_FRAME = 1;
        /// You should treat as a copy (null) frame.
        const COPY_FRAME = 2;
    }
}

/// Information for the output.
#[derive(Clone)]
#[repr(C)]
pub struct OutputInfo {
    /// Flags for the output.
    pub flag: OutputInfoFlag,
    /// The width of the video.
    pub w: c_int,
    /// The height of the video.
    pub h: c_int,
    /// The numerator of the frame rate.
    pub rate: c_int,
    /// The denominator of the frame rate.
    pub scale: c_int,
    /// The total frames of the video data.
    pub n: c_int,
    /// The byte length per frame.
    pub size: c_int,
    /// The sampling rate of the audio.
    pub audio_rate: c_int,
    /// The number of channels of the audio.
    pub audio_ch: c_int,
    /// The total samples of the audio.
    pub audio_n: c_int,
    /// The byte length per audio sample.
    pub audio_size: c_int,
    /// The save file name.
    pub save_file: LPSTR,
    /// Gets the pointer to the DIB format (RGB 24 bit) image data.
    ///
    /// # Parameters
    ///
    /// 1. The frame index.
    ///
    /// # Returns
    ///
    /// The pointer to the image. The data is valid until next API calling or returning your process to AviUtl.
    pub func_get_video: unsafe extern "system" fn(c_int) -> *mut c_void,
    /// Gets the pointer to the PCM 16 bit format audio data.
    ///
    /// # Parameters
    ///
    /// 1. The start index to get.
    /// 2. The length to get.
    /// 3. The pointer to be written how many samples read.
    ///
    /// # Returns
    ///
    /// The pointer to the waveform. The data is valid until next API calling or returning your process to AviUtl.
    pub func_get_audio: unsafe extern "system" fn(c_int, c_int, *mut c_int) -> *mut c_void,
    /// Checks whether you should abort the output.
    ///
    /// # Returns
    ///
    /// True if you should abort the output.
    pub func_is_abort: unsafe extern "system" fn() -> BOOL,
    /// Displays the remaining time.
    ///
    /// # Parameters
    ///
    /// 1. The index of the frame processing now.
    /// 2. The total frames of the output.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub func_rest_time_disp: unsafe extern "system" fn(c_int, c_int) -> BOOL,
    /// Gets the flag of the frame.
    ///
    /// # Parameters
    ///
    /// 1. The frame index.
    ///
    /// # Returns
    ///
    /// Flags of the frame.
    pub func_get_flag: unsafe extern "system" fn(c_int) -> FrameFlag,
    /// Updates the preview. The latest frame loaded by `func_get_video` will be displayed.
    ///
    /// # Returns
    ///
    /// True if succeed to update.
    pub func_update_preview: unsafe extern "system" fn() -> BOOL,
    /// Gets the DIB format image data.
    ///
    /// # Parameters
    ///
    /// 1. The frame index.
    /// 2. The image format. Specifying null will be RGB 24 bit. `u32::from_bytes([b'Y', b'U', b'Y', b'2'])` will be YUY2, and `u32::from_bytes([b'Y', b'C', b'4', b'8'])` will be YCbCr. But YCbCr is unavailable on YUY2 filter mode.
    ///
    /// # Returns
    ///
    /// The pointer to the image data. The data is valid until next API calling or returning your process to AviUtl.
    pub func_get_video_ex: unsafe extern "system" fn(c_int, u32) -> *mut c_void,
}

/// Definition of the output plugin.
///
/// # Safety
///
/// You must use only for share the table to AviUtl.
#[repr(C)]
pub struct OutputPlugin {
    /// Currently unused. Set 0.
    pub flag: c_int,
    /// The plugin name.
    pub name: LPSTR,
    /// The file filter string.
    pub file_filter: LPSTR,
    /// The plugin information.
    pub information: LPSTR,
    /// Initialization handler of the output plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_init: unsafe extern "system" fn() -> BOOL,
    /// Exiting handler of the output plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_exit: unsafe extern "system" fn() -> BOOL,
    /// Output (export) handler of the output plugin. You should not set null.
    ///
    /// # Parameters
    ///
    /// 1. The information of the output.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_output: unsafe extern "system" fn(*mut OutputInfo) -> BOOL,
    /// Configuration dialog handler of the output plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the window.
    /// 2. The DLL instance.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_config: unsafe extern "system" fn(HWND, HINSTANCE) -> BOOL,
    /// Getting configuration handler of the output plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the buffer to write the configuration.
    /// 2. The length of the buffer.
    ///
    /// # Returns
    ///
    /// The length of the data you wrote.
    pub func_config_get: unsafe extern "system" fn(*mut c_void, c_int) -> c_int,
    /// Setting configuration handler of the output plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the buffer to read the configuration.
    /// 2. The length of the buffer.
    ///
    /// # Returns
    ///
    /// The length of the data you read.
    pub func_config_set: unsafe extern "system" fn(*mut c_void, c_int) -> c_int,
    /// Reserved.
    pub _reserve: [c_int; 16],
}

unsafe impl Send for OutputPlugin {}
unsafe impl Sync for OutputPlugin {}
