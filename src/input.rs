use crate::{FileFilters, FrameRate, PixelFormat, Result};
use std::{borrow::Cow, os::raw::c_void};
use windows::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::BITMAPINFOHEADER,
    Media::Audio::WAVEFORMATEX,
};

pub use aviutl_plugin_sys::input::InputInfoFlag as InfoFlag;
pub use aviutl_plugin_sys::input::InputPluginFlag as PluginFlag;

#[non_exhaustive]
pub struct Info<'a> {
    pub flags: InfoFlag,
    pub frame_rate: FrameRate,
    pub video_frames: usize,
    pub video_formats: &'a [BITMAPINFOHEADER],
    pub audio_samples: usize,
    pub audio_formats: &'a [WAVEFORMATEX],
    pub codec: PixelFormat,
}

impl Default for Info<'_> {
    fn default() -> Self {
        Self {
            flags: InfoFlag::empty(),
            frame_rate: FrameRate { rate: 30, scale: 1 },
            video_frames: 0,
            video_formats: &[],
            audio_samples: 0,
            audio_formats: &[],
            codec: PixelFormat(0),
        }
    }
}

pub trait InputHandle: Sized {
    fn get_info(&mut self) -> Result<Info> {
        Ok(Info::default())
    }
    fn read_video(&mut self, _frame: usize, _buf: *mut c_void) -> Result<usize> {
        Ok(0)
    }
    fn read_audio(&mut self, _frame: usize, _buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
    fn is_key_frame(&mut self, _frame: usize) -> bool {
        true
    }
}

pub mod prelude {
    pub use super::{InputHandle, InputPlugin, PluginFlag};
    pub use crate::{export_input_plugin, AviUtlError, FileFilters, Result};
}

/// Input plugin (`.aui`) handlers.
pub trait InputPlugin: Default {
    type Handle: InputHandle;

    const NAME: &'static str;
    const INFORMATION: &'static str;
    const FLAGS: PluginFlag;
    fn file_filters() -> FileFilters;
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn exit(&mut self) -> Result<()> {
        Ok(())
    }
    fn open(&mut self, path: Cow<'_, str>) -> Result<Self::Handle>;
    fn close(&mut self, handle: Self::Handle) -> Result<()>;
    fn config_dialog(&mut self, _window: HWND, _dll: HINSTANCE) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! export_input_plugin {
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
        static TABLE: ::once_cell::sync::Lazy<::aviutl_plugin_sys::input::InputPlugin> =
            ::once_cell::sync::Lazy::new(|| ::aviutl_plugin_sys::input::InputPlugin {
                flag: <$impl>::FLAGS,
                name: NAME.as_ptr() as _,
                file_filter: FILE_FILTER.as_ptr() as _,
                information: INFORMATION.as_ptr() as _,
                func_init,
                func_exit,
                func_open,
                func_close,
                func_info_get,
                func_read_video,
                func_read_audio,
                func_is_keyframe,
                func_config,
                _reserve: [0; 16],
            });
        #[no_mangle]
        unsafe extern "system" fn GetInputPluginTable(
        ) -> *const ::aviutl_plugin_sys::input::InputPlugin {
            &*TABLE
        }
        unsafe extern "system" fn func_init() -> i32 {
            PLUGIN.lock().unwrap().init().is_ok() as i32
        }
        unsafe extern "system" fn func_exit() -> i32 {
            PLUGIN.lock().unwrap().exit().is_ok() as i32
        }
        unsafe extern "system" fn func_open(file_name: *mut u8) -> *mut ::std::os::raw::c_void {
            let (cow, _, had_error) = ::encoding_rs::SHIFT_JIS.decode(unsafe {
                ::std::ffi::CStr::from_ptr(file_name as *const _ as *const ::std::os::raw::c_char)
                    .to_bytes()
            });
            if had_error {
                return ::std::ptr::null_mut();
            }
            let handle = PLUGIN.lock().unwrap().open(cow);
            match handle {
                Ok(handle) => ::std::boxed::Box::into_raw(::std::boxed::Box::new(handle)) as *mut _,
                Err(_) => ::std::ptr::null_mut(),
            }
        }
        unsafe extern "system" fn func_close(handle: *mut ::std::os::raw::c_void) -> i32 {
            let handle = unsafe {
                Box::<<$impl as InputPlugin>::Handle>::from_raw(
                    handle as *mut <$impl as InputPlugin>::Handle,
                )
            };
            PLUGIN.lock().unwrap().close(*handle).is_ok() as i32
        }
        unsafe extern "system" fn func_info_get(
            handle: *mut ::std::os::raw::c_void,
            info: *mut ::aviutl_plugin_sys::input::InputInfo,
        ) -> i32 {
            let handle = unsafe { &mut *(handle as *mut <$impl as InputPlugin>::Handle) };
            let wrapped = handle.get_info();
            if wrapped.is_err() {
                return 0;
            }
            let wrapped = wrapped.unwrap();
            let info = unsafe { &mut *info };
            info.flag = wrapped.flags;
            info.rate = wrapped.frame_rate.rate as _;
            info.scale = wrapped.frame_rate.scale as _;
            info.n = wrapped.video_frames as _;
            info.format = wrapped.video_formats.as_ptr().cast();
            info.format_size = wrapped.video_formats.len() as _;
            info.audio_n = wrapped.audio_samples as _;
            info.audio_format = wrapped.audio_formats.as_ptr().cast();
            info.audio_format_size = wrapped.audio_formats.len() as _;
            info.handler = wrapped.codec.into_raw();
            1
        }
        unsafe extern "system" fn func_read_video(
            handle: *mut ::std::os::raw::c_void,
            frame: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int {
            let handle = unsafe { &mut *(handle as *mut <$impl as InputPlugin>::Handle) };
            handle.read_video(frame as _, buf).unwrap_or(0) as _
        }
        unsafe extern "system" fn func_read_audio(
            handle: *mut ::std::os::raw::c_void,
            frame: ::std::os::raw::c_int,
            len: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int {
            let handle = unsafe { &mut *(handle as *mut <$impl as InputPlugin>::Handle) };
            unsafe {
                handle
                    .read_audio(
                        frame as _,
                        ::std::slice::from_raw_parts_mut(buf.cast(), len as _),
                    )
                    .unwrap_or(0) as _
            }
        }
        unsafe extern "system" fn func_is_keyframe(
            handle: *mut ::std::os::raw::c_void,
            frame: ::std::os::raw::c_int,
        ) -> i32 {
            let handle = unsafe { &mut *(handle as *mut <$impl as InputPlugin>::Handle) };
            handle.is_key_frame(frame as _) as _
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
    };
}
