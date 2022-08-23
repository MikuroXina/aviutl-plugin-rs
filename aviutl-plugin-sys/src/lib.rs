//! This sys crate provides FFI data definition for AviUtl Plugin DLL (Win32).
#![warn(missing_docs)]

use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::os::raw::{c_int, c_short, c_void};

/// YCbCr pixel data. These values may go out from its range.
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
pub struct PixelYc {
    /// Luma data, between 0 and 4096.
    pub y: c_short,
    /// Blue-difference data, between -2048 and 2048.
    pub cb: c_short,
    /// Red-difference data, between -2048 and 2048.
    pub cr: c_short,
}

/// Definition of multi thread function callback.
///
/// # Parameters
///
/// 1. `thread_id` - The current id of the thread, between 0 and thread_num.
/// 2. `thread_num` - The number of threads.
/// 3. `param1` - A generic parameter 1.
/// 4. `param2` - A generic parameter 2.
pub type MultiThreadFunc = unsafe extern "system" fn(c_int, c_int, *mut c_void, *mut c_void);

pub mod color;
pub mod filter;
pub mod input;
pub mod output;
