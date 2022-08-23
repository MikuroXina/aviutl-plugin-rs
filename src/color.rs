use crate::{AviUtlError, MultiThreadFn, PixelFormat, Result, Size};
use aviutl_plugin_sys::{color::ColorProcInfo, MultiThreadFunc, PixelYc};
use std::{
    marker::PhantomData,
    os::raw::{c_int, c_void},
};

pub use aviutl_plugin_sys::color::ColorProcInfoFlag as ProcInfoFlag;

pub struct ProcInfo<'a> {
    pub flags: ProcInfoFlag,
    pub format: PixelFormat,
    pub size: Size,
    pub line_bytes: usize,
    exec_multi_thread_func:
        unsafe extern "system" fn(MultiThreadFunc, *mut c_void, *mut c_void) -> i32,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ProcInfo<'a> {
    pub fn from_raw(raw: &'a ColorProcInfo) -> Self {
        let format = PixelFormat::from_four_code(raw.format.to_le_bytes());
        let size = Size {
            width: raw.w as _,
            height: raw.h as _,
        };
        Self {
            flags: raw.flag,
            format,
            size,
            line_bytes: raw.line_size as _,
            exec_multi_thread_func: raw.exec_multi_thread_func,
            _phantom: PhantomData,
        }
    }

    pub fn exec_multi_thread_func<T: MultiThreadFn + 'a>(&self, func: &T) -> Result<()> {
        unsafe extern "system" fn wrapped<T: MultiThreadFn>(
            id: c_int,
            num: c_int,
            closure: *mut c_void,
            _: *mut c_void,
        ) {
            (*(closure as *const T))(id as usize, num as usize);
        }
        let res = unsafe {
            (self.exec_multi_thread_func)(
                wrapped::<T>,
                func as *const T as *mut _,
                std::ptr::null_mut(),
            )
        };
        if res == 0 {
            Err(AviUtlError::ThreadExecutionFailure)
        } else {
            Ok(())
        }
    }
}

pub mod prelude {
    pub use super::{ColorPlugin, ProcInfo};
    pub use crate::{export_color_plugin, PixelYc, Result};
}

pub trait ColorPlugin: Default {
    const NAME: &'static str;
    const INFORMATION: &'static str;
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn exit(&mut self) -> Result<()> {
        Ok(())
    }
    fn pixel_to_yc(
        &mut self,
        _proc_info: &ProcInfo,
        _from: &[u8],
        _to: &mut [PixelYc],
    ) -> Result<()> {
        Ok(())
    }
    fn yc_to_pixel(
        &mut self,
        _proc_info: &ProcInfo,
        _from: &[PixelYc],
        _to: &mut [u8],
    ) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! export_color_plugin {
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
        static INFORMATION: ::once_cell::sync::Lazy<&'static ::std::ffi::CStr> =
            ::once_cell::sync::Lazy::new(|| {
                Box::leak(
                    ::std::ffi::CString::new(<$impl>::INFORMATION)
                        .unwrap()
                        .into_boxed_c_str(),
                )
            });
        static TABLE: ::once_cell::sync::Lazy<::aviutl_plugin_sys::color::ColorPlugin> =
            ::once_cell::sync::Lazy::new(|| ::aviutl_plugin_sys::color::ColorPlugin {
                flag: 0,
                name: NAME.as_ptr() as _,
                information: INFORMATION.as_ptr() as _,
                func_init,
                func_exit,
                func_pixel2yc,
                func_yc2pixel,
                _reserve: [0; 16],
            });
        #[no_mangle]
        unsafe extern "system" fn GetColorPluginTable(
        ) -> *const ::aviutl_plugin_sys::color::ColorPlugin {
            &*TABLE
        }
        unsafe extern "system" fn func_init() -> i32 {
            PLUGIN.lock().unwrap().init().is_ok() as _
        }
        unsafe extern "system" fn func_exit() -> i32 {
            PLUGIN.lock().unwrap().exit().is_ok() as _
        }
        unsafe extern "system" fn func_pixel2yc(
            proc_info: *mut ::aviutl_plugin_sys::color::ColorProcInfo,
        ) -> i32 {
            let proc_info = unsafe { &*proc_info };
            let yc_p_len = proc_info.yc_size as usize;
            let wrapped = unsafe { ::aviutl_plugin::color::ProcInfo::from_raw(proc_info) };
            let pixel_p_len = wrapped.format.bytes_per_pixel() * wrapped.size.area();
            unsafe {
                PLUGIN
                    .lock()
                    .unwrap()
                    .pixel_to_yc(
                        &wrapped,
                        ::std::slice::from_raw_parts(proc_info.pixel_p.cast(), pixel_p_len),
                        ::std::slice::from_raw_parts_mut(proc_info.yc_p.cast(), yc_p_len),
                    )
                    .is_ok() as _
            }
        }
        unsafe extern "system" fn func_yc2pixel(
            proc_info: *mut ::aviutl_plugin_sys::color::ColorProcInfo,
        ) -> i32 {
            let proc_info = unsafe { &*proc_info };
            let yc_p_len = proc_info.yc_size as usize;
            let wrapped = unsafe { ::aviutl_plugin::color::ProcInfo::from_raw(proc_info) };
            let pixel_p_len = wrapped.format.bytes_per_pixel() * wrapped.size.area();
            unsafe {
                PLUGIN
                    .lock()
                    .unwrap()
                    .yc_to_pixel(
                        &wrapped,
                        ::std::slice::from_raw_parts(proc_info.yc_p.cast(), yc_p_len),
                        ::std::slice::from_raw_parts_mut(proc_info.pixel_p.cast(), pixel_p_len),
                    )
                    .is_ok() as _
            }
        }
    };
}
