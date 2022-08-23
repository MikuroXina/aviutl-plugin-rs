//! This library provides exporter for AviUtl plugin function table, and wraps AviUtl API in Rust conventions.

// #![warn(missing_docs)]

use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    io,
    os::raw::c_char,
};
use thiserror::Error;

pub use aviutl_plugin_sys::filter::Pixel as PixelRgb;
pub use aviutl_plugin_sys::PixelYc;

/// Error in AviUtl plugin system, but not handled strictly.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AviUtlError {
    #[error("buffer limit exceeded in AviUtl")]
    BufferLimitExceed,
    #[error("windows error")]
    Windows(#[from] windows::core::Error),
    #[error("load error")]
    Load(io::Error),
    #[error("save error")]
    Save(io::Error),
    #[error("file error")]
    File(io::Error),
    #[error("invalid path")]
    InvalidPath(String),
    #[error("failed to update AviUtl main window")]
    FailedUpdateMainWindow,
    #[error("the feature {0} was unsupported")]
    Unsupported(String),
    #[error("execution in aviutl threads failed")]
    ThreadExecutionFailure,
    #[error("failed to configure {0}")]
    ConfigFailure(String),
    #[error("frame {0} is out of range")]
    FrameIndexOutOfRange(usize),
}

/// Result type of an AviUtl plugin.
pub type Result<T> = std::result::Result<T, AviUtlError>;

pub(crate) fn into_win_str(string: &'_ str) -> Cow<'_, [u8]> {
    let (cow, _, had_error) = encoding_rs::SHIFT_JIS.encode(string);
    if had_error {
        panic!("must not contains null byte");
    }
    cow
}

pub(crate) fn from_win_str(string: &'_ [u8]) -> Cow<'_, str> {
    let (cow, _, had_error) = encoding_rs::SHIFT_JIS.decode(string);
    if had_error {
        panic!("expected Shift-JIS");
    }
    cow
}

pub(crate) unsafe fn from_nullable_lpstr(lpstr: *mut u8) -> Option<String> {
    if lpstr.is_null() {
        None
    } else {
        let cstr = CStr::from_ptr(lpstr as *const _ as *const c_char);
        Some(from_win_str(cstr.to_bytes()).into_owned())
    }
}

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
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub const fn clamp(self, min: i32, max: i32) -> Self {
        const fn clamp_const(value: i32, min: i32, max: i32) -> i32 {
            debug_assert!(min <= max);
            if value < min {
                min
            } else if value > max {
                max
            } else {
                value
            }
        }
        Self {
            x: clamp_const(self.x, min, max),
            y: clamp_const(self.y, min, max),
        }
    }
}

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
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }

    pub const fn area(self) -> usize {
        self.width as usize * self.height as usize
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub size: Size,
    pub point: Point,
}

impl Rect {
    pub const fn new() -> Self {
        Self {
            size: Size::new(),
            point: Point::new(),
        }
    }

    pub const fn left(self) -> i32 {
        self.point.x
    }

    pub const fn right(self) -> i32 {
        self.point.x + self.size.width as i32
    }

    pub const fn top(self) -> i32 {
        self.point.y
    }

    pub const fn bottom(self) -> i32 {
        self.point.y + self.size.height as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
    pub rate: u32,
    pub scale: u32,
}

impl FrameRate {
    pub fn as_f32(self) -> f32 {
        self.as_f64() as f32
    }

    pub fn as_f64(self) -> f64 {
        self.rate as f64 / self.scale as f64
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelFormat(u32);

impl PixelFormat {
    pub fn rgb24() -> Self {
        Self(0)
    }

    pub fn yuy2() -> Self {
        Self::from_four_code([b'Y', b'U', b'Y', b'2'])
    }

    pub fn from_four_code(code: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(code))
    }

    pub fn bytes_per_pixel(self) -> usize {
        todo!()
    }

    pub fn into_raw(self) -> u32 {
        self.0
    }
}

pub trait MultiThreadFn: Fn(usize, usize) + Send {}

impl<F: Fn(usize, usize) + Send> MultiThreadFn for F {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileFilter {
    pub name: String,
    pub blob_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileFilters(Vec<FileFilter>);

impl FileFilters {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn add_filter(&mut self, name: impl Into<String>, blob_pattern: impl Into<String>) {
        self.0.push(FileFilter {
            name: name.into(),
            blob_pattern: blob_pattern.into(),
        });
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn to_c_string(&self) -> CString {
        let strings: Vec<_> = self
            .0
            .iter()
            .cloned()
            .map(|FileFilter { name, blob_pattern }| format!("{}\0{}", name, blob_pattern))
            .collect();
        let mut joined = strings.join("\0");
        joined.push('\0');
        unsafe { CString::from_vec_with_nul_unchecked(joined.into_bytes()) }
    }
}

impl Default for FileFilters {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for FileFilters {
    type Item = FileFilter;
    type IntoIter = std::vec::IntoIter<FileFilter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a FileFilters {
    type Item = &'a FileFilter;
    type IntoIter = std::slice::Iter<'a, FileFilter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub mod color;
pub mod filter;
pub mod input;
pub mod output;

// TODO: add prelude
