use crate::{from_win_str, AviUtlError, FileFilters, FrameRate, PixelFormat, Result, Size};
use aviutl_plugin_sys::output::OutputInfo;
use std::{borrow::Cow, ffi::CStr, ops::Range, ptr::NonNull};
use windows::Win32::Foundation::{HINSTANCE, HWND};

pub use aviutl_plugin_sys::output::FrameFlag;
pub use aviutl_plugin_sys::output::OutputInfoFlag as InfoFlag;

#[derive(Debug)]
pub struct Info<'a> {
    pub flag: InfoFlag,
    pub size: Size,
    pub frame_rate: FrameRate,
    pub video_frames: usize,
    pub video_bytes_per_frame: usize,
    pub audio_sample_rate: u32,
    pub audio_channels: usize,
    pub audio_samples: usize,
    pub audio_bytes_per_sample: usize,
    pub save_file: Cow<'a, str>,
    raw: NonNull<OutputInfo>,
}

impl<'a> Info<'a> {
    /// Converts the raw pointer of [`OutputInfo`] into a wrapped [`Info`].
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, aligned, and valid. Otherwise it occurs UB.
    pub unsafe fn from_raw(ptr: *mut OutputInfo) -> Self {
        debug_assert!(!ptr.is_null());
        let info_ref = &*ptr;
        Self {
            flag: info_ref.flag,
            size: Size {
                width: info_ref.w.unsigned_abs(),
                height: info_ref.h.unsigned_abs(),
            },
            frame_rate: FrameRate {
                rate: info_ref.rate.unsigned_abs(),
                scale: info_ref.scale.unsigned_abs(),
            },
            video_frames: info_ref.n.try_into().unwrap(),
            video_bytes_per_frame: info_ref.size.try_into().unwrap(),
            audio_sample_rate: info_ref.audio_rate.try_into().unwrap(),
            audio_channels: info_ref.audio_ch.try_into().unwrap(),
            audio_samples: info_ref.audio_ch.try_into().unwrap(),
            audio_bytes_per_sample: info_ref.audio_size.try_into().unwrap(),
            save_file: from_win_str(CStr::from_ptr(info_ref.save_file as *const _).to_bytes()),
            raw: NonNull::new(ptr).unwrap(),
        }
    }

    pub fn get_video(&self, frame: usize) -> &[u8] {
        unsafe {
            let ptr = (self.raw.as_ref().func_get_video)(frame as _);
            std::slice::from_raw_parts(ptr.cast(), self.video_bytes_per_frame)
        }
    }

    pub fn get_video_ex(&self, frame: usize, format: PixelFormat) -> &[u8] {
        unsafe {
            let ptr = (self.raw.as_ref().func_get_video_ex)(frame as i32, format.0);
            std::slice::from_raw_parts(ptr.cast(), format.bytes_per_pixel() * self.size.area())
        }
    }

    pub fn get_audio(&self, range: Range<usize>) -> &[u8] {
        let mut written_samples = 0;
        unsafe {
            let ptr = (self.raw.as_ref().func_get_audio)(
                range.start as _,
                range.end.saturating_sub(range.start) as _,
                &mut written_samples,
            );
            std::slice::from_raw_parts(ptr.cast(), written_samples as usize)
        }
    }

    pub fn is_aborted(&self) -> bool {
        unsafe { (self.raw.as_ref().func_is_abort)() != 0 }
    }

    pub fn show_remaining_time(&self, current: usize, total: usize) -> Result<()> {
        let res = unsafe { (self.raw.as_ref().func_rest_time_disp)(current as _, total as _) };
        if res != 0 {
            Ok(())
        } else {
            Err(AviUtlError::FailedUpdateMainWindow)
        }
    }

    pub fn get_flag(&self, frame: usize) -> FrameFlag {
        unsafe { (self.raw.as_ref().func_get_flag)(frame as _) }
    }

    pub fn update_preview(&self) -> Result<()> {
        let res = unsafe { (self.raw.as_ref().func_update_preview)() };
        if res != 0 {
            Ok(())
        } else {
            Err(AviUtlError::FailedUpdateMainWindow)
        }
    }
}

pub mod prelude {
    pub use super::{Info, OutputPlugin};
    pub use crate::{export_output_plugin, AviUtlError, FileFilters, Result};
}

pub trait OutputPlugin: Default {
    const NAME: &'static str;
    const INFORMATION: &'static str;
    fn file_filters() -> FileFilters;
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn exit(&mut self) -> Result<()> {
        Ok(())
    }
    fn output(&mut self, info: Info) -> Result<()>;
    fn config_dialog(&mut self, _window: HWND, _dll: HINSTANCE) -> Result<()> {
        Ok(())
    }
    fn config_get(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
    fn config_set(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
}

#[macro_export]
macro_rules! export_output_plugin {
    ($impl:ty) => {
        static PLUGIN: ::once_cell::sync::Lazy<::std::sync::Mutex<$impl>> =
            ::once_cell::sync::Lazy::new(|| <$impl>::default().into());
        static NAME: ::once_cell::sync::Lazy<&'static ::std::ffi::CStr> =
            ::once_cell::sync::Lazy::new(|| {
                Box::leak(
                    ::std::ffi::CString::new(<$impl>::NAME)
                        .unwrap()
                        .into_boxed_c_str(),
                )
            });
        static FILE_FILTER: ::once_cell::sync::Lazy<&'static ::std::ffi::CStr> =
            ::once_cell::sync::Lazy::new(|| {
                ::std::boxed::Box::leak(<$impl>::file_filters().to_c_string().into_boxed_c_str())
            });
        static INFORMATION: ::once_cell::sync::Lazy<&'static ::std::ffi::CStr> =
            ::once_cell::sync::Lazy::new(|| {
                Box::leak(
                    ::std::ffi::CString::new(<$impl>::INFORMATION)
                        .unwrap()
                        .into_boxed_c_str(),
                )
            });
        static TABLE: ::once_cell::sync::Lazy<::aviutl_plugin_sys::output::OutputPlugin> =
            ::once_cell::sync::Lazy::new(|| ::aviutl_plugin_sys::output::OutputPlugin {
                flag: 0,
                name: NAME.as_ptr() as _,
                file_filter: FILE_FILTER.as_ptr() as _,
                information: INFORMATION.as_ptr() as _,
                func_init,
                func_exit,
                func_output,
                func_config,
                func_config_get,
                func_config_set,
                _reserve: [0; 16],
            });
        #[no_mangle]
        unsafe extern "system" fn GetOutputPluginTable(
        ) -> *const ::aviutl_plugin_sys::output::OutputPlugin {
            &*TABLE
        }
        unsafe extern "system" fn func_init() -> i32 {
            PLUGIN.lock().unwrap().init().is_ok() as i32
        }
        unsafe extern "system" fn func_exit() -> i32 {
            PLUGIN.lock().unwrap().exit().is_ok() as i32
        }
        unsafe extern "system" fn func_output(
            output_info: *mut ::aviutl_plugin_sys::output::OutputInfo,
        ) -> i32 {
            let info = unsafe { ::aviutl_plugin::output::Info::from_raw(output_info) };
            PLUGIN.lock().unwrap().output(info).is_ok() as i32
        }
        unsafe extern "system" fn func_config(window: isize, dll: isize) -> i32 {
            PLUGIN
                .lock()
                .unwrap()
                .config_dialog(
                    ::windows::Win32::Foundation::HWND(window),
                    ::windows::Win32::Foundation::HINSTANCE(dll),
                )
                .is_ok() as i32
        }
        unsafe extern "system" fn func_config_get(
            data: *mut ::std::os::raw::c_void,
            n: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int {
            let slice = unsafe { ::std::slice::from_raw_parts_mut(data.cast(), n as usize) };
            PLUGIN.lock().unwrap().config_get(slice).unwrap_or(0) as _
        }
        unsafe extern "system" fn func_config_set(
            data: *mut ::std::os::raw::c_void,
            n: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int {
            let slice = unsafe { ::std::slice::from_raw_parts_mut(data.cast(), n as usize) };
            PLUGIN.lock().unwrap().config_set(slice).unwrap_or(0) as _
        }
    };
}
