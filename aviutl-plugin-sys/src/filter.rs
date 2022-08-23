//! From filter plugin header for AviUtl version 0.99k or later by ＫＥＮくん.

use crate::{MultiThreadFunc, PixelYc};
use bitflags::bitflags;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::os::raw::{c_int, c_short, c_uchar, c_void};
use windows_sys::{
    core::PSTR as LPSTR,
    Win32::{
        Foundation::{BOOL, HINSTANCE, HWND, LPARAM, WPARAM},
        Graphics::Gdi::HFONT,
        UI::WindowsAndMessaging::WM_USER,
    },
};

/// BGR pixel data.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
)]
#[repr(C)]
pub struct Pixel {
    /// Blue data, between 0 and 255.
    pub b: c_uchar,
    /// Green data, between 0 and 255.
    pub g: c_uchar,
    /// Red data, between 0 and 255.
    pub r: c_uchar,
}

/// Handle of open editing file such as a project file.
pub type EditingHandle = *mut c_void;

/// Information for `func_proc` event handler in [`Filter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FilterProcInfo {
    /// Flags of the filter event handler.
    pub flag: FilterProcInfoFlag,
    /// The pointer to the image data. This can be swapped with `yc_p_temp`.
    pub yc_p_edit: *mut PixelYc,
    /// The pointer to the temporary image data. This can be swapped with `yc_p_edit`.
    pub yc_p_temp: *mut PixelYc,
    /// The width of the image.
    pub w: c_int,
    /// The height of the image.
    pub h: c_int,
    /// The width of the editable region.
    pub max_w: c_int,
    /// The height of the editable region.
    pub max_h: c_int,
    /// The number of the frame which starts from 0.
    pub frame: c_int,
    /// The total numbers of the frames.
    pub frame_n: c_int,
    /// The width of the original image.
    pub org_w: c_int,
    /// The height of the original image.
    pub org_h: c_int,
    /// The pointer to the audio data (only valid if this filter is for audio).
    ///
    /// The format is PCM 16 bit. If the audio is stereo, the audio sample of left channel and right channel come alternately (the length will be twice).
    pub audio_p: *mut c_short,
    /// The total number of the audio samples.
    pub audio_n: c_int,
    /// The number of the audio channels.
    pub audio_ch: c_int,
    /// Unused.
    pub _pixel_p: *const Pixel,
    /// The extended editor handle.
    pub edit_p: EditingHandle,
    /// The number of bytes in the editable region.
    pub yc_size: c_int,
    /// The number of bytes in the width of the editable region.
    pub line_size: c_int,
    /// Reserved.
    pub _reserve: [c_int; 8],
}

bitflags! {
    /// Flag for the filter event handler.
    pub struct FilterProcInfoFlag: c_int {
        /// The fields in a pixel will be inverted.
        const INVERT_FIELD_ORDER = 0x0001_0000;
        /// The way of de-interlacing will be inverted.
        const INVERT_INTERLACE = 0x0002_0000;
    }
}

/// The status of a frame.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FrameStatus {
    /// The identifier number of an actual video.
    pub video: c_int,
    /// The identifier number of an actual audio.
    pub audio: c_int,
    /// The interlace mode of a frame.
    pub inter: FrameInterlace,
    /// Unused.
    pub index24fps: c_int,
    /// The identifier of the profile environment of a frame.
    pub config: c_int,
    /// The identifier of the video compression manager of a frame.
    pub vcm: c_int,
    /// The frame flag given by editor.
    pub edit_flag: EditFlag,
    /// Reserved.
    pub _reserve: [c_int; 9],
}

/// A interlace mode of a frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameInterlace {
    /// Normal frame.
    #[default]
    Normal = 0,
    /// Reversed frame.
    Reverse = 1,
    /// Odd (top) field.
    Odd = 2,
    /// Even (bottom) field.
    Even = 3,
    /// Mixed frame.
    Mix = 4,
    /// Auto configured.
    Auto = 5,
}

bitflags! {
    /// The flag of a frame.
    pub struct EditFlag: c_int {
        /// Treated as a keyframe on encoding.
        const KEYFRAME = 1;
        /// Marked in the editor.
        const MARK_FRAME = 2;
        /// Preferred as a non-keyframe on encoding.
        const DEL_FRAME = 4;
        /// Treated as a copy frame, which shows previous one.
        const NULL_FRAME = 8;
    }
}

/// Information for the open file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FileInfo {
    /// Flags of the file.
    pub flag: FileInfoFlag,
    /// The file name. It will be null on came from `avi_file_open`.
    pub name: LPSTR,
    /// Thw width of the image.
    pub w: c_int,
    /// Thw height of the image.
    pub h: c_int,
    /// The numerator of the frame rate.
    pub video_rate: c_int,
    /// The denominator of the frame rate.
    pub video_scale: c_int,
    /// The sampling rate of the audio.
    pub audio_rate: c_int,
    /// The total number of the audio channels.
    pub audio_ch: c_int,
    /// The total number of the video frames.
    pub frame_n: c_int,
    /// The decoding format of the video.
    pub video_decode_format: u32,
    /// The bits of the decode format.
    pub video_decode_bit: c_int,
    /// The total number of the audio samples. Valid only came from `avi_file_open`.
    pub audio_n: c_int,
    /// Reserved.
    pub _reserve: [c_int; 4],
}

bitflags! {
    /// Flag of the open file.
    pub struct FileInfoFlag: c_int {
        /// It contains some video track.
        const VIDEO = 1;
        /// It contains some audio track.
        const AUDIO = 2;
    }
}

/// Information of the AviUtl system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SysInfo {
    /// Flags of the system.
    pub flag: SysInfoFlag,
    /// Version info.
    pub info: LPSTR,
    /// The number of registered filters.
    pub filter_n: c_int,
    /// The minimum width of the image which can be edited.
    pub min_w: c_int,
    /// The minimum height of the image which can be edited.
    pub min_h: c_int,
    /// The maximum width of the image which can be edited.
    pub max_w: c_int,
    /// The maximum height of the image which can be edited.
    pub max_h: c_int,
    /// The maximum number of the frames which can be edited.
    pub max_frame: c_int,
    /// The file name of editing now, which will be null when no file name.
    pub edit_name: LPSTR,
    /// The open project file name, which will be null when no file name.
    pub project_name: LPSTR,
    /// The output file name, which will be null when no file name.
    pub output_name: LPSTR,
    /// The width of the editing region.
    pub vram_w: c_int,
    /// The height of the editing region.
    pub vram_h: c_int,
    /// The number of bytes in the editing region.
    pub vram_yc_size: c_int,
    /// The number of bytes in the width of the editing region.
    pub vram_line_size: c_int,
    /// The font handle used by the filter configuration window.
    pub font_handle: HFONT,
    /// The build revision number, will be bigger on newer version.
    pub build: c_int,
    /// Reserved.
    pub _reserve: [c_int; 2],
}

bitflags! {
    /// Flag of the AviUtl system.
    pub struct SysInfoFlag: c_int {
        /// Editing now.
        const EDIT = 1;
        /// VFAPI activated.
        const VFAPI = 2;
        /// Using SSE.
        const USE_SSE = 4;
        /// Using SSE2.
        const USE_SSE2 = 8;
    }
}

/// The handle of inputted AVI file.
pub type AviFileHandle = *mut c_void;

bitflags! {
    /// Flag for open the media file.
    pub struct FileOpenFlag: c_int {
        /// Loads only the video data.
        const VIDEO_ONLY = 0x10;
        /// Loads only the audio data.
        const AUDIO_ONLY = 0x20;
        /// Decodes with YUY2 pixel format.
        const ONLY_YUY2 = 0x1_0000;
        /// Decodes with RGB24 pixel format.
        const ONLY_RGB24 = 0x2_0000;
        /// Decodes with RGB32 pixel format.
        const ONLY_RGB32 = 0x4_0000;
    }
}

/// Status type of the frame stored in the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameStatusType {
    /// Edit flags of the frame.
    EditFlag = 0,
    /// Interlace mode of the frame.
    Interlace = 1,
}

/// Type of the file filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FileFilterType {
    /// The video filter.
    Video = 0,
    /// The audio filter.
    Audio = 1,
}

bitflags! {
    /// Modifier key flags for the shortcut key of a menu item.
    pub struct AddMenuItemFlagKey: c_int {
        /// Adds shift key modifier.
        const SHIFT = 1;
        /// Adds control key modifier.
        const CTRL = 2;
        /// Adds alternate key modifier.
        const ALT = 4;
    }
}

bitflags! {
    /// Flag for opening an editing file.
    pub struct EditOpenFlag: c_int {
        /// Loads additionally.
        const ADD = 0x2;
        /// Loads an audio file.
        const AUDIO = 0x10;
        /// Opens an AviUtl project file.
        const PROJECT = 0x200;
        /// Displays the open dialog.
        const DIALOG = 0x1_0000;
    }
}

bitflags! {
    /// Flags for exporting an editing file.
    pub struct EditOutputFlag: c_int {
        /// Not to display the export dialog.
        const NO_DIALOG = 2;
        /// Use WAV export.
        const WAV = 4;
    }
}

/// API function table exported from AviUtl.
#[repr(C)]
pub struct Exports {
    /// Prefer to use `get_yc_p_source_cache` over this.
    ///
    /// Gets the pointer to the image data before filter of the frame moved by the specified offset frames on the AVI file.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The offset from the index.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null on failure. The data is valid until next API calling or returning your process to AviUtl.
    #[deprecated = "You should use `get_yc_p_source_cache`"]
    pub get_yc_p_ofs: unsafe extern "system" fn(EditingHandle, c_int, c_int) -> *mut c_void,
    /// Prefer to use `get_yc_p_source_cache` over this.
    ///
    /// Gets the pointer to the image data before filter of the frame on the AVI file.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null on failure. The data is valid until next API calling or returning your process to AviUtl.
    #[deprecated = "You should use `get_yc_p_source_cache`"]
    pub get_yc_p: unsafe extern "system" fn(EditingHandle, c_int) -> *mut c_void,
    /// Gets the pointer to the image data before filter on specified frame by DIB format (RGB 24 bit).
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// The pointer to the DIB format image data, or null on failure. The data is valid until next API calling or returning your process to AviUtl.
    pub get_pixel_p: unsafe extern "system" fn(EditingHandle, c_int) -> *mut c_void,
    /// Gets the audio data before filter on specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The buffer to be stored, or null if you need only number of samples.
    ///
    /// # Returns
    ///
    /// The number of samples read.
    pub get_audio: unsafe extern "system" fn(EditingHandle, c_int, *mut c_void) -> c_int,
    /// Checks whether the user is editing.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    ///
    /// # Returns
    ///
    /// Whether the user is editing.
    pub is_editing: unsafe extern "system" fn(EditingHandle) -> BOOL,
    /// Checks whether the user is saving.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    ///
    /// # Returns
    ///
    /// Whether the user is saving.
    pub is_saving: unsafe extern "system" fn(EditingHandle) -> BOOL,
    /// Gets the frame index showing now.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    ///
    /// # Returns
    ///
    /// Current frame index.
    pub get_frame: unsafe extern "system" fn(EditingHandle) -> c_int,
    /// Gets the total number of the frames.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    ///
    /// # Returns
    ///
    /// Total number of the frames.
    pub get_frame_n: unsafe extern "system" fn(EditingHandle) -> c_int,
    /// Gets the size of the frame before filter.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to be stored the width.
    /// 3. The pointer to be stored the height.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub get_frame_size: unsafe extern "system" fn(EditingHandle, *mut c_int, *mut c_int) -> BOOL,
    /// Sets the frame index showing now.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. New frame index to be set.
    ///
    /// # Returns
    ///
    /// Configured frame index.
    pub set_frame: unsafe extern "system" fn(EditingHandle, c_int) -> c_int,
    /// Sets the total number of the frames.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. New total number to be set.
    ///
    /// # Returns
    ///
    /// Configured total number of the frames.
    pub set_frame_n: unsafe extern "system" fn(EditingHandle, c_int) -> c_int,
    /// Copies both video and audio of the frame into another frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. Destination frame index.
    /// 3. Source frame index.
    ///
    /// # Returns
    ///
    /// True if succeed to copy.
    pub copy_frame: unsafe extern "system" fn(EditingHandle, c_int, c_int) -> BOOL,
    /// Copies video of the frame into another frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. Destination frame index.
    /// 3. Source frame index.
    ///
    /// # Returns
    ///
    /// True if succeed to copy.
    pub copy_video: unsafe extern "system" fn(EditingHandle, c_int, c_int) -> BOOL,
    /// Copies audio of the frame into another frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. Destination frame index.
    /// 3. Source frame index.
    ///
    /// # Returns
    ///
    /// True if succeed to copy.
    pub copy_audio: unsafe extern "system" fn(EditingHandle, c_int, c_int) -> BOOL,
    /// Copies the current frame into clipboard by DIB format (RGB 24 bit).
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to the DIB format image data.
    /// 3. The width of the image.
    /// 4. The width of the height.
    ///
    /// # Returns
    ///
    /// True if succeed to copy.
    pub copy_clip: unsafe extern "system" fn(HWND, *mut c_void, c_int, c_int) -> BOOL,
    /// Pastes clipboard into the specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the window.
    /// 2. The handle of the editor.
    /// 3. The frame index.
    ///
    /// # Returns
    ///
    /// True if succeed to paste.
    pub paste_clip: unsafe extern "system" fn(HWND, *mut c_void, c_int) -> BOOL,
    /// Gets the frame status.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to be written the frame status.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_frame_status: unsafe extern "system" fn(EditingHandle, c_int, *mut FrameStatus) -> BOOL,
    /// Sets the frame status.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to the frame status will be stored.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub set_frame_status: unsafe extern "system" fn(EditingHandle, c_int, *mut FrameStatus) -> BOOL,
    /// Checks whether the frame will be saved actually.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// Whether the frame will be saved.
    pub is_saveframe: unsafe extern "system" fn(EditingHandle, c_int) -> BOOL,
    /// Checks whether the frame is a keyframe.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// Whether the frame is a keyframe.
    pub is_keyframe: unsafe extern "system" fn(EditingHandle, c_int) -> BOOL,
    /// Checks whether the frame needs recompression.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// Whether the frame needs recompression.
    pub is_recompress: unsafe extern "system" fn(EditingHandle, c_int) -> BOOL,
    /// Rerenders the track bars and checkboxes in the configuration window.
    ///
    /// # Parameters
    ///
    /// 1. The pointe to the filter.
    ///
    /// # Returns
    ///
    /// True if succeed to rerender.
    pub filter_window_update: unsafe extern "system" fn(*mut FilterPlugin) -> BOOL,
    /// Checks whether the filter configuration window is displaying now.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    ///
    /// # Returns
    ///
    /// True if it is displaying.
    pub is_filter_window_disp: unsafe extern "system" fn(*mut FilterPlugin) -> BOOL,
    /// Gets the information of the edit file.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to be written the file information.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_file_info: unsafe extern "system" fn(EditingHandle, *mut FileInfo) -> BOOL,
    /// Gets the name of the current profile.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The id number of the profile environment.
    ///
    /// # Returns
    ///
    /// The pointer to the name of the profile, or null if failed.
    pub get_config_name: unsafe extern "system" fn(EditingHandle, c_int) -> LPSTR,
    /// Checks whether the filter is active.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    ///
    /// # Returns
    ///
    /// Whether the filter is active.
    pub is_filter_active: unsafe extern "system" fn(*mut FilterPlugin) -> BOOL,
    /// Gets the image data before filter on specified frame in DIB format (RGB 24 bit).
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to store the DIB format data, or ignored if specified null.
    /// 4. The pointer to store the width of the image, or ignored if specified null.
    /// 5. The pointer to store the height of the image, or ignored if specified null.
    ///
    /// # Returns
    ///
    /// True if succeed.
    pub get_pixel_filtered: unsafe extern "system" fn(
        EditingHandle,
        c_int,
        *mut c_void,
        *mut c_int,
        *mut c_int,
    ) -> BOOL,
    /// Gets the audio data after filter on specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to the buffer to be written the audio, or ignored if specified null.
    ///
    /// # Returns
    ///
    /// The number of the samples read.
    pub get_audio_filtered: unsafe extern "system" fn(EditingHandle, c_int, *mut c_void) -> c_int,
    /// Gets the range of the selected frames.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to be written start frame of the selection.
    /// 3. The pointer to be written end frame of the selection.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_select_frame: unsafe extern "system" fn(EditingHandle, *mut c_int, *mut c_int) -> BOOL,
    /// Sets the range of the selected frames.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The start frame of the selection.
    /// 3. The end frame of the selection.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub set_select_frame: unsafe extern "system" fn(EditingHandle, c_int, c_int) -> BOOL,
    /// Converts RGB pixels into YCbCr pixels.
    ///
    /// # Parameters
    ///
    /// 1. The first pointer to sequence of YCbCr pixels buffer.
    /// 2. The first pointer to sequence of RGB pixels buffer.
    /// 3. The length of the buffer.
    ///
    /// # Returns
    ///
    /// True if succeed to convert.
    pub rgb2yc: unsafe extern "system" fn(*mut PixelYc, *const Pixel, c_int) -> BOOL,
    /// Converts YCbCr pixels into RGB pixels.
    ///
    /// # Parameters
    ///
    /// 1. The first pointer to sequence of RGB pixels buffer.
    /// 2. The first pointer to sequence of YCbCr pixels buffer.
    /// 3. The length of the buffer.
    ///
    /// # Returns
    ///
    /// True if succeed to convert.
    pub yc2rgb: unsafe extern "system" fn(*mut Pixel, *const PixelYc, c_int) -> BOOL,
    /// Gets the file name to load by file dialog.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to be stored the file name.
    /// 2. The file filtering string.
    /// 3. The default file name.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if cancelled.
    pub dlg_get_load_name: unsafe extern "system" fn(LPSTR, LPSTR, LPSTR) -> BOOL,
    /// Gets the file name to save by file dialog.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to be stored the file name.
    /// 2. The file filtering string.
    /// 3. The default file name.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if cancelled.
    pub dlg_get_save_name: unsafe extern "system" fn(LPSTR, LPSTR, LPSTR) -> BOOL,
    /// Reads an integer from the ini file.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The key name to access.
    /// 3. The default value if no value exists.
    ///
    /// # Returns
    ///
    /// The integer read from the ini file.
    pub ini_load_int: unsafe extern "system" fn(*mut FilterPlugin, LPSTR, c_int) -> c_int,
    /// Writes an integer to the ini file.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The key name to access.
    /// 3. The value to write.
    ///
    /// # Returns
    ///
    /// The integer written to the ini file.
    pub ini_save_int: unsafe extern "system" fn(*mut FilterPlugin, LPSTR, c_int) -> c_int,
    /// Reads a string from the ini file.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The key name to access.
    /// 3. The buffer to be stored the string.
    /// 4. The default value if no value exists.
    ///
    /// # Returns
    ///
    /// True if succeed to read.
    pub ini_load_str: unsafe extern "system" fn(*mut FilterPlugin, LPSTR, LPSTR, LPSTR) -> BOOL,
    /// Writes a string to the ini file.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The key name to access.
    /// 3. The string to write.
    ///
    /// # Returns
    ///
    /// True if succeed to read.
    pub ini_save_str: unsafe extern "system" fn(*mut FilterPlugin, LPSTR, LPSTR) -> BOOL,
    /// Gets the file information of the specified file id.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to be written the file information.
    /// 3. The file id.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_source_file_info:
        unsafe extern "system" fn(EditingHandle, *mut FileInfo, c_int) -> BOOL,
    /// Gets the file id of source of the specified frame and its source video id.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to be written the file id.
    /// 4. The pointer to be written the source video id.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_source_video_number:
        unsafe extern "system" fn(EditingHandle, c_int, *mut c_int, *mut c_int) -> BOOL,
    /// Gets the system information.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The pointer to be written the system information.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_sys_info: unsafe extern "system" fn(EditingHandle, *mut SysInfo) -> BOOL,
    /// Gets the pointer to the filter by specified id.
    ///
    /// # Parameters
    ///
    /// 1. The filter id, between 0 and the number of registered filters.
    ///
    /// # Returns
    ///
    /// The pointer to the filter, or null if failed.
    pub get_filter_p: unsafe extern "system" fn(c_int) -> *mut FilterPlugin,
    /// Gets the pointer to the image data filtering on specified frame. The frame is filtered until this filter.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The handle of the editor.
    /// 3. The frame index.
    /// 4. Reserved. Specify null.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null if failed. The data is valid until next API calling or returning your process to AviUtl.
    #[deprecated = "You should use `get_yc_p_filtering_cache_ex`"]
    pub get_yc_p_filtering: unsafe extern "system" fn(
        *mut FilterPlugin,
        EditingHandle,
        c_int,
        *const c_void,
    ) -> *mut c_void,
    /// Gets the audio data on specified frame. The frame is filtered until this filter.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The handle of the editor.
    /// 3. The frame index.
    /// 4. The buffer to be stored, or ignored if specified null.
    ///
    /// # Returns
    ///
    /// The number of the samples read.
    pub get_audio_filtering:
        unsafe extern "system" fn(*mut FilterPlugin, EditingHandle, c_int, *mut c_void) -> u32,
    /// Configures the cache behaviour in `get_yc_p_filtering_cache_ex`. The cache region is reallocated only when changes found, and allocated only when the filter is active.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The width of the cache region.
    /// 3. The height of the cache region.
    /// 4. The number of the frames to cache.
    /// 5. Reserved. Specify null.
    ///
    /// # Returns
    ///
    /// True if succeed to configure.
    pub set_yc_p_filtering_cache_size:
        unsafe extern "system" fn(*mut FilterPlugin, c_int, c_int, c_int, c_int) -> BOOL,
    /// Gets the pointer to the cached image data filtering on specified frame. The frame is filtered until this filter.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The handle of the editor.
    /// 3. The frame index.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null if failed. The data is valid until next API calling or returning your process to AviUtl.
    #[deprecated = "You should use `get_yc_p_filtering_cache_ex`"]
    pub get_yc_p_filtering_cache:
        unsafe extern "system" fn(*mut FilterPlugin, EditingHandle, c_int) -> *mut c_void,
    /// Gets the pointer to the cached image data before filter on specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The frame offset on original AVI data.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null if failed. The data is valid until dropped from the cache.
    pub get_yc_p_source_cache:
        unsafe extern "system" fn(EditingHandle, c_int, c_int) -> *mut c_void,
    /// Gets the pointer to the filtered image data which is displaying now. Available only if filter is visible.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The image format. Specifying null will be RGB 24 bit. `u32::from_bytes([b'Y', b'U', b'Y', b'2'])` will be YUY2.
    ///
    /// # Returns
    ///
    /// The pointer to the filtered image data. The data is valid until next API calling or returning your process to AviUtl.
    pub get_disp_pixel_p: unsafe extern "system" fn(EditingHandle, u32) -> *mut c_void,
    /// Gets the image data before filter on specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to be written the DIB format data.
    /// 4. The image format. Specifying null will be RGB 24 bit. `u32::from_bytes([b'Y', b'U', b'Y', b'2'])` will be YUY2.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_pixel_source: unsafe extern "system" fn(EditingHandle, c_int, *mut c_void, u32) -> BOOL,
    /// Gets the image data and size before filter on specified frame.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The frame index.
    /// 3. The pointer to be written the DIB format data.
    /// 4. The pointer to be written the width of the image.
    /// 5. The pointer to be written the height of the image.
    /// 6. The image format. Specifying null will be RGB 24 bit. `u32::from_bytes([b'Y', b'U', b'Y', b'2'])` will be YUY2.
    ///
    /// # Returns
    ///
    /// True if succeed to get.
    pub get_pixel_filtered_ex: unsafe extern "system" fn(
        EditingHandle,
        c_int,
        *mut c_void,
        *mut c_int,
        *mut c_int,
        u32,
    ) -> BOOL,
    /// Gets the pointer to the cached image data and size on specified frame. The frame is filtered until this filter. Cache behaviour can be configured by `set_yc_p_filtering_cache_size`.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The handle of the editor.
    /// 3. The frame index.
    /// 4. The pointer to be written the width of the image.
    /// 5. The pointer to be written the height of the image.
    ///
    /// # Returns
    ///
    /// The pointer to the image data, or null if failed. The data is valid until dropped from the cache.
    pub get_yc_p_filtering_cache_ex: unsafe extern "system" fn(
        *mut FilterPlugin,
        EditingHandle,
        c_int,
        *mut c_int,
        *mut c_int,
    ) -> *mut PixelYc,
    /// Invokes the multi thread function with numbers of thread configured in AviUtl preferences. Do not call AviUtl or Win32 API in the function because its thread differs from the filter event handler's.
    ///
    /// # Parameters
    ///
    /// 1. The function to be invoked in other threads.
    /// 2. Generic parameter 1.
    /// 3. Generic parameter 2.
    ///
    /// # Returns
    ///
    /// True if succeed to setup invocation.
    pub exec_multi_thread_func:
        unsafe extern "system" fn(MultiThreadFunc, *mut c_void, *mut c_void) -> BOOL,
    /// Creates the empty frame, image data region. This pointer must not swap with `yc_p_edit` or `yc_p_temp` in [`FilterProcInfo`]. You should drop the created frame by `delete_yc` on exiting AviUtl.
    ///
    /// # Returns
    ///
    /// The pointer to the created frame.
    pub create_yc: unsafe extern "system" fn() -> *mut PixelYc,
    /// Deallocates the frame, image data region.
    ///
    /// # Parameters
    ///
    /// 1. The frame to deallocate.
    pub delete_yc: unsafe extern "system" fn(*mut PixelYc),
    /// Loads the BMP file into the frame image data.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the frame image to load into.
    /// 2. The BMP file name to load.
    /// 3. The pointer to be written the width of the image, or ignored if specified null.
    /// 4. The pointer to be written the height of the image, or ignored if specified null.
    /// 5. Reserved. Specify null.
    ///
    /// # Returns
    ///
    /// True if succeed to load.
    pub load_image:
        unsafe extern "system" fn(*mut PixelYc, LPSTR, *mut c_int, *mut c_int, c_int) -> BOOL,
    /// Resizes the frame image data clopped by area.
    ///
    /// # Parameters
    ///
    /// 1. Destination pointer to be written the resized frame image.
    /// 2. The width to resize.
    /// 3. The height to resize.
    /// 4. Source pointer to the frame image. Specifying null will be same as the first argument.
    /// 5. The x coordinate of clopping area in the source image.
    /// 6. The y coordinate of clopping area in the source image.
    /// 7. The width of clopping area in the source image.
    /// 8. The height of clopping area in the source image.
    pub resize_yc: unsafe extern "system" fn(
        *mut PixelYc,
        c_int,
        c_int,
        *const PixelYc,
        c_int,
        c_int,
        c_int,
        c_int,
    ),
    /// Copies the frame image data clopped by area. The drawn image clipped by the maximum size in AviUtl preferences. Do not overlap the region of source and destination, or occurs UB.
    ///
    /// # Parameters
    ///
    /// 1. Destination pointer to be written the resized frame image.
    /// 2. The width to resize.
    /// 3. The height to resize.
    /// 4. Source pointer to the frame image.
    /// 5. The x coordinate of clopping area in the source image.
    /// 6. The y coordinate of clopping area in the source image.
    /// 7. The width of clopping area in the source image.
    /// 8. The height of clopping area in the source image.
    /// 9. The opacity of clopping, between 0 and 4096.
    pub copy_yc: unsafe extern "system" fn(
        *mut PixelYc,
        c_int,
        c_int,
        *const PixelYc,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
    ),
    /// Renders the text into the frame image data. The drawn image clipped by the maximum size in AviUtl preferences.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the frame image data to be rendered. Specifying null will calculate only the size of the rendered text.
    /// 2. The x coordinate of top left point to render.
    /// 3. The y coordinate of top left point to render.
    /// 4. The text to draw.
    /// 5. The r component of color to render, between 0 and 255.
    /// 6. The g component of color to render, between 0 and 255.
    /// 7. The b component of color to render, between 0 and 255.
    /// 8. The opacity to render, between 0 and 4096.
    /// 9. The font used in rendering. Specifying null will be the default font.
    /// 10. The pointer to be written the width of rendered region, or ignored if specified null.
    /// 11. The pointer to be written the height of rendered region, or ignored if specified null.
    pub draw_text: unsafe extern "system" fn(
        *mut PixelYc,
        c_int,
        c_int,
        LPSTR,
        c_int,
        c_int,
        c_int,
        c_int,
        HFONT,
        *mut c_int,
        *mut c_int,
    ),
    /// Opens the media file (not only AVI file), then returns the handle for usage of `avi_file_read_video`, `avi_file_read_audio`, and so on. Note that the file or format may differ from the editing file's.
    ///
    /// # Parameters
    ///
    /// 1. The media file name to load. You can also specify the file format supported in other input plugins.
    /// 2. The pointer to be written the file information,
    /// 3. Flag for opening the file.
    ///
    /// # Returns
    ///
    /// The open file handle.
    pub avi_file_open:
        unsafe extern "system" fn(LPSTR, *mut FileInfo, FileOpenFlag) -> AviFileHandle,
    /// Closes the file handle.
    ///
    /// # Parameters
    ///
    /// 1. The file handle to close.
    pub avi_file_close: unsafe extern "system" fn(AviFileHandle),
    /// Loads an image data into specified frame from the file.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the file to read.
    /// 2. The pointer to the frame image to write.
    /// 3. The frame index.
    ///
    /// # Returns
    ///
    /// True if succeed to read.
    pub avi_file_read_video: unsafe extern "system" fn(AviFileHandle, *mut PixelYc, c_int) -> BOOL,
    /// Loads an audio data into specified frame from the file.
    ///
    /// # Parameters
    ///
    /// 1. The file handle to read.
    /// 2. The pointer to the audio buffer to write.
    /// 3. The frame index.
    ///
    /// # Returns
    ///
    /// The number of the audio samples read.
    pub avi_file_read_audio: unsafe extern "system" fn(AviFileHandle, *mut c_void, c_int) -> c_int,
    /// Gets the pointer to the DIB format image data from the file. The format will be as specified in `avi_file_open`.
    ///
    /// # Parameters
    ///
    /// 1. The file handle to read.
    /// 2. The frame index.
    ///
    /// # Returns
    ///
    /// The pointer to the DIB format image data, or null if failed. The data is valid until next API calling or returning your process to AviUtl.
    pub avi_file_get_video_pixel_p:
        unsafe extern "system" fn(AviFileHandle, c_int) -> *const c_void,
    /// Gets the file filter which used on loading the file type.
    ///
    /// # Parameters
    ///
    /// 1. Type of the file.
    ///
    /// # Returns
    ///
    /// The pointer to the file filter.
    pub get_avi_file_filter: unsafe extern "system" fn(FileFilterType) -> LPSTR,
    /// Loads the audio samples from the file.
    ///
    /// # Parameters
    ///
    /// 1. The file handler to read.
    /// 2. The index of samples to start reading from.
    /// 3. The number of samples to read.
    /// 4. The pointer to the buffer to load into.
    ///
    /// # Returns
    ///
    /// The number of samples read.
    pub avi_file_read_audio_sample:
        unsafe extern "system" fn(AviFileHandle, c_int, c_int, *mut c_void) -> c_int,
    /// Configures the sample rate for loading by `avi_file_read_audio_sample`.
    ///
    /// # Parameters
    ///
    /// 1. The file handle to configure.
    /// 2. The audio sample rate to set.
    /// 3. The number of audio channels.
    ///
    /// # Returns
    ///
    /// The total number of samples on changed sampling rate.
    pub avi_file_set_audio_sample_rate:
        unsafe extern "system" fn(AviFileHandle, c_int, c_int) -> c_int,
    /// Gets the pointer to the frame status buffer.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    /// 2. The status type.
    ///
    /// # Returns
    ///
    /// The pointer to the buffer, valid until closed the editing file.
    pub get_frame_status_table:
        unsafe extern "system" fn(EditingHandle, FrameStatusType) -> *const u8,
    /// Sets the current editing situation into the undo buffer.
    ///
    /// # Parameters
    ///
    /// 1. The handle of the editor.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub set_undo: unsafe extern "system" fn(EditingHandle) -> BOOL,
    /// Adds the menu item into the settings menu in the main window. When the menu item is selected, `COMMAND` message will be sent to the specified window. This function must be called in `func_init` or the initialization handler.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to the filter.
    /// 2. The name of the menu item.
    /// 3. The window handle to be sent the `COMMAND` message.
    /// 4. The id of the menu item, sent to the handler.
    /// 5. The default shortcut key code of the menu item, or null if none.
    /// 6. Flags for the modifier key of the shortcut.
    ///
    /// # Returns
    ///
    /// True if succeed to add.
    pub add_menu_item: unsafe extern "system" fn(
        *mut FilterPlugin,
        LPSTR,
        HWND,
        c_int,
        c_int,
        AddMenuItemFlagKey,
    ) -> BOOL,
    /// Opens the editing file.
    ///
    /// # Parameters
    ///
    /// 1. The editing handle.
    /// 2. The file name to open.
    /// 3. Flags to open file.
    ///
    /// # Returns
    ///
    /// True if succeed to open.
    pub edit_open: unsafe extern "system" fn(EditingHandle, LPSTR, EditOpenFlag) -> BOOL,
    /// Closes the editing file.
    ///
    /// # Parameters
    ///
    /// 1. The editing handle.
    ///
    /// # Returns
    ///
    /// True if succeed to close.
    pub edit_close: unsafe extern "system" fn(EditingHandle) -> BOOL,
    /// Exports the editing file by the output plugin.
    ///
    /// # Parameters
    ///
    /// 1. The editing handle.
    /// 2. The file name to output.
    /// 3. Flags for output the file.
    /// 4. The output plugin name to use.
    ///
    /// # Returns
    ///
    /// True if succeed to export.
    pub edit_output: unsafe extern "system" fn(EditingHandle, LPSTR, EditOutputFlag, LPSTR) -> BOOL,
    /// Sets the profile into the editing file.
    ///
    /// # Parameters
    ///
    /// 1. The editing handle.
    /// 2. The number of the profile environment to set.
    /// 3. The profile name to set.
    ///
    /// # Returns
    ///
    /// True if succeed to set.
    pub set_config: unsafe extern "system" fn(EditingHandle, c_int, LPSTR) -> BOOL,
    /// Reserved.
    pub _reserve: [c_int; 7],
}

bitflags! {
    /// Flag for filter definition.
    pub struct FilterFlag: c_int {
        /// The filter is active currently.
        const ACTIVE = 0x1;
        /// The filter will always be active.
        const ALWAYS_ACTIVE = 0x4;
        /// The plugin item in the settings menu will be a popup menu.
        const CONFIG_POPUP = 0x8;
        /// The plugin item in the settings menu will be a checkbox.
        const CONFIG_CHECK = 0x10;
        /// The plugin item in the settings menu will be a radio button.
        const CONFIG_RADIO = 0x20;
        /// Enables the extended data to be able to save.
        const EX_DATA = 0x400;
        /// The priority of the filter make the highest.
        const PRIORITY_HIGHEST = 0x800;
        /// The priority of the filter make the lowest.
        const PRIORITY_LOWEST = 0x1000;
        /// The plugin window will be resizable.
        const WINDOW_THICK_FRAME = 0x2000;
        /// The plugin window will be sizable by itself.
        const WINDOW_SIZE = 0x4000;
        /// The size data will be client size of the window.
        const WINDOW_SIZE_CLIENT = FilterFlag::WINDOW_SIZE.bits | 0x1000_0000;
        /// The size data will be relative to the original size.
        const WINDOW_SIZE_ADD = FilterFlag::WINDOW_SIZE.bits | 0x3000_0000;
        /// The filter item is visible from the user.
        const DISP_FILTER = 0x8000;
        /// Whether redrawing is controller by the plugin.
        const REDRAW = 0x2_0000;
        /// The extended information of the filter can be set.
        const EX_INFORMATION = 0x4_0000;
        /// The information of the filter can be set.
        #[deprecated = "You should use `EX_INFORMATION`"]
        const INFORMATION = 0x80000;
        /// Disables the window of the plugin.
        const NO_CONFIG = 0x100000;
        /// This plugin is also an audio filter.
        const AUDIO_FILTER = 0x200000;
        /// Converts the checkboxes into the radio buttons in the plugin window.
        const RADIO_BUTTON = 0x400000;
        /// Append the horizontal scroll bar in the plugin window.
        const WINDOW_HORIZONTAL_SCROLL = 0x800000;
        /// Append the vertical scroll bar in the plugin window.
        const WINDOW_VERTICAL_SCROLL = 0x1000000;
        /// This plugin is also a de-interlacing filter.
        const INTERLACE_FILTER = 0x4000000;
        /// Stops to create the initial data for `func_proc`.
        const NO_INIT_DATA = 0x8000000;
        /// Adds the import menu item of the plugin.
        const IMPORT = 0x10000000;
        /// Adds the export menu item of the plugin.
        const EXPORT = 0x20000000;
        /// Whether `MAIN` messages will be sent to `func_window_proc`.
        const MAIN_MESSAGE = 0x40000000;
    }
}

/// An update event of the filter configuration.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FilterUpdateStatus(pub c_int);

impl FilterUpdateStatus {
    /// All items have changed.
    pub const ALL: Self = Self(0);
    /// The track bar has changed. Lower byte is the changed index.
    pub const TRACK: Self = Self(0x1_0000);
    /// The checkbox has changed. Lower byte is the changed index.
    pub const CHECK: Self = Self(0x2_0000);

    /// Checks whether the flag belongs to TRACK status.
    pub fn is_track(self) -> bool {
        self.contains(Self::TRACK)
    }
    /// Checks whether the flag belongs to CHECK status.
    pub fn is_check(self) -> bool {
        self.contains(Self::CHECK)
    }

    /// Checks whether the flag bit set contains the other flag bit set.
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Gets lower bits of the status.
    pub fn lower_bits(self) -> c_int {
        self.0 as u8 as c_int
    }
}

/// An extended window message sent to the plugin window.
///
/// # Extra parameters
///
/// ## `WPARAM`
///
/// - `Command`: `MID_FILTER_BUTTON` plus the index of pushed button.
/// - `MainMouseWheel`: Scroll amount in upper word.
/// - `MainKey*`: Virtual key code.
///
/// ## `LPARAM`
///
/// - `MainMouse*`: The coordinate on the editing image. X coordinate in lower word, and Y coordinate in higher word.
/// - `MainMoveSize`: The window handle of main window.
/// - `MainContextMenu`: The coordinate on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u32)]
pub enum WindowMessage {
    /// Filter configuration or editing data was changed.
    Update = WM_USER + 100,
    /// Editing file was opened.
    FileOpen = WM_USER + 101,
    /// Editing file was closed.
    FileClose = WM_USER + 102,
    /// Post initialization of the plugin.
    Init = WM_USER + 103,
    /// Pre exiting of the plugin.
    Exit = WM_USER + 104,
    /// Pre saving the file.
    SaveStart = WM_USER + 105,
    /// Post saving the file.
    SaveEnd = WM_USER + 106,
    /// Selected to import.
    Import = WM_USER + 107,
    /// Selected to export.
    Export = WM_USER + 108,
    /// Activated/deactivated the filter plugin.
    ChangeActive = WM_USER + 109,
    /// Showed/hided the plugin window.
    ChangeWindow = WM_USER + 110,
    /// Changed the parameters of the plugin.
    ChangeParam = WM_USER + 111,
    /// Changed the whether it is editing or not.
    ChangeEdit = WM_USER + 112,
    /// Emitted when `index`-th button was pressed.
    Command = WM_USER + 113,
    /// Appended the file such as additional movie, audio, and so on.
    FileUpdate = WM_USER + 114,
    /// The left button of mouse was clicked in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseDown = WM_USER + 120,
    /// The left button of mouse was released in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseUp = WM_USER + 121,
    /// Mouse was moved in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseMove = WM_USER + 122,
    /// Key is pressed in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainKeyDown = WM_USER + 123,
    /// Key is released in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainKeyUp = WM_USER + 124,
    /// The size or position was changed in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMoveSize = WM_USER + 125,
    /// The left button of mouse was double-clicked in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseDoubleClick = WM_USER + 126,
    /// The right button of mouse was clicked in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseRightDown = WM_USER + 127,
    /// The right button of mouse was released in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseRightUp = WM_USER + 128,
    /// The wheel of mouse was scrolled in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required.
    MainMouseWheel = WM_USER + 129,
    /// The context menu was triggered in main window. [`FilterFlag::MAIN_MESSAGE`] flag is required. Note that you should return `true` from the handler.
    MainContextMenu = WM_USER + 130,
}

/// Shift value for WPARAM to emit a message that the button in the plugin window was clicked. See also [`WindowMessage`].
pub const MID_FILTER_BUTTON: WPARAM = 12004;

/// Definition of the filter plugin.
///
/// # Safety
///
/// You must use only for share the table to AviUtl.
#[repr(C)]
pub struct FilterPlugin {
    /// Flags for the filter plugin.
    pub flag: FilterFlag,
    /// The width of the plugin window.
    pub width: c_int,
    /// The height of the plugin window.
    pub height: c_int,
    /// The name of the filter.
    pub name: LPSTR,
    /// The number of the track bars.
    pub track_n: c_int,
    /// The head pointer to the array of track names. It can be null only if `track_n` is zero.
    pub track_name: *const LPSTR,
    /// The head pointer to the array of track default values. It can be null only if `track_n` is zero.
    pub track_default: *const c_int,
    /// The head pointer to the array of track minimum values. If it is null, treated all as zeroes.
    pub track_s: *const c_int,
    /// The head pointer to the array of track maximum values. If it is null, treated all as 256s.
    pub track_e: *const c_int,
    /// The number of the checkboxes.
    pub check_n: c_int,
    /// The head pointer to the array of track names. It can be null only if `check_n` is zero.
    pub check_name: *const LPSTR,
    /// The head pointer to the array of track default values. It can be null only if `check_n` is zero. If the default value is negative, it will be a button, which send a window message `WM_COMMAND` with `WPARAM = MID_FILTER_BUTTON + index`.
    pub check_default: *const c_int,
    /// Processor handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The information for filter processor.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_proc: unsafe extern "system" fn(*mut FilterPlugin, *const FilterProcInfo) -> BOOL,
    /// Initialization handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_init: unsafe extern "system" fn(*mut FilterPlugin) -> BOOL,
    /// Exiting handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_exit: unsafe extern "system" fn(*mut FilterPlugin) -> BOOL,
    /// Configuration update handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. What the configuration was updated.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_update: unsafe extern "system" fn(*mut FilterPlugin, FilterUpdateStatus) -> BOOL,
    /// Window message handler of the filter plugin, or ignored if null. If VFAPI is activated, this will not be triggered.
    ///
    /// # Parameters
    ///
    /// 1. The plugin window handler.
    /// 2. The window message.
    /// 3. An extra parameter 1 of the window message. More details are in [`WindowMessage`].
    /// 4. An extra parameter 2 of the window message. More details are in [`WindowMessage`].
    /// 5. The editing handle.
    /// 6. The pointer to itself.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_window_proc: unsafe extern "system" fn(
        HWND,
        WindowMessage,
        WPARAM,
        LPARAM,
        EditingHandle,
        *mut FilterPlugin,
    ) -> BOOL,
    /// The current track bar values which set by AviUtl.
    pub track: *const c_int,
    /// The current checkbox values which set by AviUtl.
    pub check: *const c_int,
    /// The pointer to the extended data. Available only the plugin has [`FilterFlag::EX_DATA`] flag.
    pub ex_data_ptr: *mut c_void,
    /// The size to the extended data. Available only the plugin has [`FilterFlag::EX_DATA`] flag.
    pub ex_data_size: c_int,
    /// The pointer to the filter information.
    pub information: LPSTR,
    /// Pre saving handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The start frame of the range to save.
    /// 3. The end frame of the range to save.
    /// 4. The editing handle.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_save_start:
        unsafe extern "system" fn(*mut FilterPlugin, c_int, c_int, EditingHandle) -> BOOL,
    /// Post saving handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The editing handle.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_save_end: unsafe extern "system" fn(*mut FilterPlugin, EditingHandle) -> BOOL,
    /// Exported AviUtl API function table, set by AviUtl.
    pub ex_func: *const Exports,
    /// The plugin window handle, set by AviUtl.
    pub window_handle: HWND,
    /// The DLL instance handle, set by AviUtl.
    pub dll_instance: HINSTANCE,
    /// The pointer to be written the initial extended data, or ignored if null.
    pub ex_data_def: *mut c_void,
    /// Save frame handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The editing handle.
    /// 3. The frame index of the start of the saved range.
    /// 4. The current frame index.
    /// 5. The frame rate.
    /// 6. The frame edit flag.
    /// 7. The frame interlace mode.
    ///
    /// # Returns
    ///
    /// True if it should be saved, or false it should be thinned.
    pub func_is_saveframe: unsafe extern "system" fn(
        *mut FilterPlugin,
        EditingHandle,
        c_int,
        c_int,
        c_int,
        EditFlag,
        FrameInterlace,
    ) -> BOOL,
    /// Project load handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The editing handle.
    /// 3. The pointer to the loaded data.
    /// 4. The byte length of the loaded data.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_project_load:
        unsafe extern "system" fn(*mut FilterPlugin, EditingHandle, *const c_void, c_int) -> BOOL,
    /// Project save handler of the filter plugin, or ignored if null. The third argument may be null because AviUtl even call this handler for getting only the size of data to save.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The editing handle.
    /// 3. The pointer to write the data.
    /// 4. The pointer to write the byte length of the data.
    ///
    /// # Returns
    ///
    /// True if you have written the data, or false if none.
    pub func_project_save: unsafe extern "system" fn(
        *mut FilterPlugin,
        EditingHandle,
        *mut c_void,
        *mut c_int,
    ) -> BOOL,
    /// Window title handler of the filter plugin, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The pointer to itself.
    /// 2. The editing handle.
    /// 3. The current frame index.
    /// 4. The pointer to the buffer to write the title bar string.
    /// 5. The length of the string buffer.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_modify_title:
        unsafe extern "system" fn(*mut FilterPlugin, EditingHandle, c_int, LPSTR, c_int) -> BOOL,
    /// The sub directory path of the plugin. Available only if the plugin is placed in `plugins` directory.
    pub dll_path: LPSTR,
    /// Reserved.
    pub _reserve: [c_int; 2],
}

unsafe impl Send for FilterPlugin {}
unsafe impl Sync for FilterPlugin {}
