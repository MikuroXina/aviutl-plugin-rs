//! From output plugin header for AviUtl version 0.99h or later by ＫＥＮくん.

use crate::{MultiThreadFunc, PixelYc};
use bitflags::bitflags;
use std::os::raw::{c_int, c_void};
use windows_sys::{core::PSTR as LPSTR, Win32::Foundation::BOOL};

bitflags! {
    /// Flag for color processor.
    pub struct ColorProcInfoFlag: c_int {
        /// Inverted the vertical direction of the data in `pixel_p`.
        const INVERT_HEIGHT = 1;
        /// Using SSE.
        const USE_SSE = 256;
        /// Using SSE2.
        const USE_SSE2 = 512;
    }
}

/// Information for the color processor.
#[derive(Clone)]
#[repr(C)]
pub struct ColorProcInfo {
    /// Flags for the color processor.
    pub flag: ColorProcInfoFlag,
    /// YCBCr pixels head pointer.
    pub yc_p: *mut PixelYc,
    /// DIB format pixels head pointer.
    pub pixel_p: *mut c_void,
    /// Pixel format of `pixel_p`.
    pub format: u32,
    /// The width of the pixels.
    pub w: c_int,
    /// The height of the pixels.
    pub h: c_int,
    /// The byte length of the width of `yc_p`.
    pub line_size: c_int,
    /// The byte length of `yc_p`.
    pub yc_size: c_int,
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
    /// Reserved.
    pub _reserve: [c_int; 16],
}

/// Definition of the color plugin.
///
/// # Safety
///
/// You must use only for share the table to AviUtl.
#[repr(C)]
pub struct ColorPlugin {
    /// Currently unused. Set 0.
    pub flag: c_int,
    /// The plugin name.
    pub name: LPSTR,
    /// The plugin information.
    pub information: LPSTR,
    /// Initialization handler of the color plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_init: unsafe extern "system" fn() -> BOOL,
    /// Exiting handler of the color plugin, or ignored if null.
    ///
    /// # Returns
    ///
    /// True if succeed, or false if failed.
    pub func_exit: unsafe extern "system" fn() -> BOOL,
    /// Conversion DIB format image `pixel_p` into YCbCr image `yc_p` handler, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The information for the color processor.
    ///
    /// # Returns
    ///
    /// True if you have converted, or false otherwise.
    pub func_pixel2yc: unsafe extern "system" fn(*mut ColorProcInfo) -> BOOL,
    /// Conversion YCbCr image `yc_p` into DIB format image `pixel_p` handler, or ignored if null.
    ///
    /// # Parameters
    ///
    /// 1. The information for the color processor.
    ///
    /// # Returns
    ///
    /// True if you have converted, or false otherwise.
    pub func_yc2pixel: unsafe extern "system" fn(*mut ColorProcInfo) -> BOOL,
    /// Reserved.
    pub _reserve: [c_int; 16],
}

unsafe impl Send for ColorPlugin {}
unsafe impl Sync for ColorPlugin {}
