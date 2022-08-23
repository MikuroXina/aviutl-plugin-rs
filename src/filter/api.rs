use super::avi_file::AviFile;
use crate::{
    into_win_str, AviUtlError, FileFilters, MultiThreadFn, PixelRgb, PixelYc, Result, Size,
};
use aviutl_plugin_sys::filter::{Exports, FilterPlugin as Table};
use std::{ffi::CStr, os::raw::c_void};
use windows::Win32::Foundation::{HINSTANCE, HWND, MAX_PATH};

pub use aviutl_plugin_sys::filter::{
    AddMenuItemFlagKey as ShortcutKeyModifier, FileFilterType, FileOpenFlag,
};

pub struct Api<'a> {
    pub(crate) filter: *mut Table,
    pub(crate) exports: &'a Exports,
}

impl<'a> Api<'a> {
    /// Creates an API table from an initialized plugin table.
    ///
    /// # Safety
    ///
    /// `raw` table must be initialized by AviUtl. Otherwise it will occur UB.
    pub unsafe fn from_raw(raw: &'a Table) -> Self {
        Self {
            filter: raw as *const _ as *mut _,
            exports: &*raw.ex_func,
        }
    }

    pub fn plugin_window(&self) -> HWND {
        HWND(unsafe { &*self.filter }.window_handle)
    }
    pub fn dll_instance(&self) -> HINSTANCE {
        HINSTANCE(unsafe { &*self.filter }.dll_instance)
    }

    pub fn get_track_value(&self, index: usize) -> Option<i32> {
        let tracks = unsafe {
            let filter = &*self.filter;
            let tracks_ptr = filter.track;
            std::slice::from_raw_parts(tracks_ptr, filter.track_n as usize)
        };
        tracks.get(index).copied()
    }
    pub fn get_check_value(&self, index: usize) -> Option<bool> {
        let checks = unsafe {
            let filter = &*self.filter;
            let checks_ptr = filter.check;
            std::slice::from_raw_parts(checks_ptr, filter.check_n as usize)
        };
        checks.get(index).copied().map(|value| value != 0)
    }

    pub fn copy_into_clipboard(&self, window: HWND, dib: &[u8], size: Size) -> Result<()> {
        if unsafe {
            (self.exports.copy_clip)(
                window.0,
                dib.as_ptr() as *mut _,
                size.width as _,
                size.height as _,
            )
        } == 0
        {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(())
        }
    }

    pub fn paste_from_clipboard(
        &mut self,
        window: HWND,
        dib: &mut [u8],
        frame: usize,
    ) -> Result<()> {
        if unsafe { (self.exports.paste_clip)(window.0, dib.as_mut_ptr().cast(), frame as _) } == 0
        {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(())
        }
    }

    pub fn is_displaying_window(&self) -> bool {
        unsafe { (self.exports.is_filter_window_disp)(self.filter) != 0 }
    }

    pub fn is_active(&self) -> bool {
        unsafe { (self.exports.is_filter_active)(self.filter) != 0 }
    }

    pub fn rgb_to_yc(&self, yc: &mut [PixelYc], rgb: &[PixelRgb]) -> Result<()> {
        assert!(rgb.len() <= yc.len());
        if unsafe { (self.exports.rgb2yc)(yc.as_mut_ptr(), rgb.as_ptr(), yc.len() as _) } == 0 {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(())
        }
    }

    pub fn yc_to_rgb(&self, rgb: &mut [PixelRgb], yc: &[PixelYc]) -> Result<()> {
        assert!(yc.len() <= rgb.len());
        if unsafe { (self.exports.yc2rgb)(rgb.as_mut_ptr(), yc.as_ptr(), rgb.len() as _) } == 0 {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(())
        }
    }

    pub fn dialog_load_file(
        &self,
        file_filters: FileFilters,
        default_file_name: &str,
    ) -> Result<String> {
        let filter_cstr = file_filters.to_c_string();
        let default_cstr = into_win_str(default_file_name);
        let mut file_name = vec![0; MAX_PATH as usize];
        unsafe {
            (self.exports.dlg_get_load_name)(
                file_name.as_mut_ptr(),
                filter_cstr.as_ptr() as *mut _,
                default_cstr.as_ptr() as *mut _,
            ) != 0
        }
        .then(|| String::from_utf8_lossy(&file_name).into_owned())
        .ok_or_else(|| AviUtlError::Unsupported("dialog input".into()))
    }

    pub fn dialog_save_file(
        &self,
        file_filters: FileFilters,
        default_file_name: &str,
    ) -> Result<String> {
        let filter_cstr = file_filters.to_c_string();
        let default_cstr = into_win_str(default_file_name);
        let mut file_name = vec![0; MAX_PATH as usize];
        unsafe {
            (self.exports.dlg_get_load_name)(
                file_name.as_mut_ptr(),
                filter_cstr.as_ptr() as *mut _,
                default_cstr.as_ptr() as *mut _,
            ) != 0
        }
        .then(|| String::from_utf8_lossy(&file_name).into_owned())
        .ok_or_else(|| AviUtlError::Unsupported("dialog input".into()))
    }

    pub fn load_int_from_ini<K: AsRef<str>>(&mut self, key: K, default: i32) -> i32 {
        let key = into_win_str(key.as_ref());
        unsafe { (self.exports.ini_load_int)(self.filter, key.as_ptr() as *mut _, default) }
    }

    pub fn save_int_into_ini<K: AsRef<str>>(&mut self, key: K, value: i32) -> i32 {
        let key = into_win_str(key.as_ref());
        unsafe { (self.exports.ini_save_int)(self.filter, key.as_ptr() as *mut _, value) }
    }

    pub fn load_str_from_ini<K: AsRef<str>, D: AsRef<str>>(
        &mut self,
        key: K,
        default: D,
    ) -> Result<String> {
        let key = into_win_str(key.as_ref());
        let mut buf = vec![0; 1024];
        let default = into_win_str(default.as_ref());
        if unsafe {
            (self.exports.ini_load_str)(
                self.filter,
                key.as_ptr() as *mut _,
                buf.as_mut_ptr(),
                default.as_ptr() as *mut _,
            ) == 0
        } {
            Err(AviUtlError::Load(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to load str from ini",
            )))
        } else {
            Ok(String::from_utf8_lossy(&buf).into_owned())
        }
    }

    pub fn save_str_into_ini<K: AsRef<str>, V: AsRef<str>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<()> {
        let key = into_win_str(key.as_ref());
        let value = into_win_str(value.as_ref());
        if unsafe {
            (self.exports.ini_save_str)(
                self.filter,
                key.as_ptr() as *mut _,
                value.as_ptr() as *mut _,
            ) == 0
        } {
            Err(AviUtlError::Load(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to save str into ini",
            )))
        } else {
            Ok(())
        }
    }

    pub fn get_filter_raw(&self, id: usize) -> *mut c_void {
        unsafe { (self.exports.get_filter_p)(id as _).cast() }
    }

    pub fn set_yc_cache_size(&mut self, size: Size, frames: usize) -> Result<()> {
        if unsafe {
            (self.exports.set_yc_p_filtering_cache_size)(
                self.filter,
                size.width as _,
                size.height as _,
                frames as _,
                0,
            )
        } == 0
        {
            Err(AviUtlError::Unsupported("setting cache size".into()))
        } else {
            Ok(())
        }
    }

    pub fn exec_multi_thread_func<T: MultiThreadFn + 'a>(&'a self, func: &T) -> Result<()> {
        unsafe extern "system" fn wrapped<T: MultiThreadFn>(
            id: ::std::os::raw::c_int,
            num: ::std::os::raw::c_int,
            closure: *mut c_void,
            _: *mut c_void,
        ) {
            (*(closure as *const T))(id as usize, num as usize);
        }
        if unsafe {
            (self.exports.exec_multi_thread_func)(
                wrapped::<T>,
                func as *const T as *mut _,
                std::ptr::null_mut(),
            )
        } == 0
        {
            Err(AviUtlError::ThreadExecutionFailure)
        } else {
            Ok(())
        }
    }

    pub fn open_avi(&self, file_name: &str, open_flag: FileOpenFlag) -> Result<AviFile> {
        AviFile::new(self, file_name, open_flag)
    }

    pub fn file_filter(&self, filter_type: FileFilterType) -> FileFilters {
        let cstr =
            unsafe { CStr::from_ptr((self.exports.get_avi_file_filter)(filter_type).cast()) };
        let components: Vec<_> = cstr.to_bytes().split(|&ch| ch == 0).collect();
        let mut filters = FileFilters::new();
        for chunk in components.chunks_exact(2) {
            filters.add_filter(
                String::from_utf8_lossy(chunk[0]),
                String::from_utf8_lossy(chunk[1]),
            );
        }
        filters
    }

    pub fn add_menu_item(
        &self,
        name: &str,
        plugin_window: HWND,
        id: usize,
        shortcut: Option<ShortcutKey>,
    ) -> Result<()> {
        let name = into_win_str(name);
        let ShortcutKey { key_code, modifier } = shortcut.unwrap_or_default();
        if unsafe {
            (self.exports.add_menu_item)(
                self.filter,
                name.as_ptr() as *mut _,
                plugin_window.0,
                id as _,
                key_code as _,
                modifier,
            )
        } == 0
        {
            Err(AviUtlError::Unsupported("adding menu item".into()))
        } else {
            Ok(())
        }
    }

    pub fn update_filter_window(&self) -> Result<()> {
        if unsafe { (self.exports.filter_window_update)(self.filter) } == 0 {
            Err(AviUtlError::Unsupported("updating filter window".into()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortcutKey {
    pub key_code: u8,
    pub modifier: ShortcutKeyModifier,
}

impl Default for ShortcutKey {
    fn default() -> Self {
        Self {
            key_code: 0,
            modifier: ShortcutKeyModifier::empty(),
        }
    }
}
