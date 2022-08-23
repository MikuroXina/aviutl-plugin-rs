use self::{api::Api, editing::Editing, window_message::WindowMessage};
use crate::{PixelYc, Result, Size};
use aviutl_plugin_sys::filter::{FilterProcInfo, FilterUpdateStatus};
use std::ops::RangeInclusive;
use windows::Win32::Foundation::{HINSTANCE, HWND};

pub use aviutl_plugin_sys::filter::FilterFlag as FilterPluginFlag;
pub use aviutl_plugin_sys::filter::FilterProcInfoFlag as ProcInfoFlag;
pub use aviutl_plugin_sys::filter::{EditFlag, FrameInterlace};

pub mod api;
pub mod avi_file;
pub mod editing;
pub mod file_info;
pub mod frame_status;
pub mod sys_info;
pub mod window_message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoId(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Track {
    pub name: &'static str,
    pub default_value: i32,
    pub min_value: i32,
    pub max_value: i32,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            name: "Untitled",
            default_value: 0,
            min_value: -64,
            max_value: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Control {
    pub name: &'static str,
    pub default_checked: bool,
    pub is_button: bool,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            name: "Untitled",
            default_checked: false,
            is_button: false,
        }
    }
}

pub struct AudioBuffer<'a> {
    data: &'a mut [i16],
    total_samples: usize,
    channels: usize,
}

impl<'a> AudioBuffer<'a> {
    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn samples_per_channel(&self) -> usize {
        self.total_samples / self.channels
    }

    pub fn samples_by_channel(&mut self, channel: usize) -> impl Iterator<Item = &mut i16> {
        let channels = self.channels;
        self.data
            .iter_mut()
            .enumerate()
            .filter(move |&(i, _)| i % channels == channel)
            .map(|(_, sample)| sample)
    }
}

pub trait Frame {
    fn image(&self) -> &[PixelYc];
    fn image_mut(&mut self) -> &mut [PixelYc];
    fn frame_size(&self) -> Size;

    fn lines(&self) -> std::slice::ChunksExact<PixelYc> {
        let Size { width, .. } = self.frame_size();
        self.image().chunks_exact(width as usize)
    }

    fn lines_mut(&mut self) -> std::slice::ChunksExactMut<PixelYc> {
        let Size { width, .. } = self.frame_size();
        self.image_mut().chunks_exact_mut(width as usize)
    }

    fn split_at_y(&mut self, y: usize) -> (BorrowedMutFrame, BorrowedMutFrame) {
        let Size { width, height } = self.frame_size();
        assert!((0..(height as usize)).contains(&y));
        let pos = y * width as usize;
        let (left, right) = self.image_mut().split_at_mut(pos);
        let left_height = y as u32;
        let right_height = height - left_height;
        (
            BorrowedMutFrame {
                image: left,
                size: Size {
                    width,
                    height: left_height,
                },
            },
            BorrowedMutFrame {
                image: right,
                size: Size {
                    width,
                    height: right_height,
                },
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedFrame {
    image: Vec<PixelYc>,
    size: Size,
}

impl OwnedFrame {
    pub fn new(size: Size) -> Self {
        Self {
            image: vec![PixelYc::default(); size.area()],
            size,
        }
    }

    pub fn borrow_mut(&mut self) -> BorrowedMutFrame {
        BorrowedMutFrame {
            image: &mut self.image,
            size: self.size,
        }
    }
}

impl Frame for OwnedFrame {
    fn image(&self) -> &[PixelYc] {
        &self.image
    }

    fn image_mut(&mut self) -> &mut [PixelYc] {
        &mut self.image
    }

    fn frame_size(&self) -> Size {
        self.size
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BorrowedMutFrame<'a> {
    image: &'a mut [PixelYc],
    size: Size,
}

impl<'a> BorrowedMutFrame<'a> {
    pub(crate) unsafe fn from_raw(image: *mut PixelYc, size: Size) -> Self {
        Self {
            image: std::slice::from_raw_parts_mut(image, size.area()),
            size,
        }
    }
}

impl Frame for BorrowedMutFrame<'_> {
    fn image(&self) -> &[PixelYc] {
        self.image
    }

    fn image_mut(&mut self) -> &mut [PixelYc] {
        self.image
    }

    fn frame_size(&self) -> Size {
        self.size
    }
}

pub struct ProcInfo<'a> {
    pub flags: ProcInfoFlag,
    pub yc_p_edit: BorrowedMutFrame<'a>,
    pub yc_p_temp: BorrowedMutFrame<'a>,
    pub size: Size,
    pub max_size: Size,
    pub current_frame: usize,
    pub total_frames: usize,
    pub original_size: Size,
    pub audio_buffer: AudioBuffer<'a>,
    pub editing: Editing<'a>,
}

impl<'a> ProcInfo<'a> {
    /// Creates wrapped information from the raw one.
    ///
    /// # Safety
    ///
    /// `raw` must be valid and received from AviUtl.
    pub unsafe fn from_raw(raw: &FilterProcInfo, api: &'a Api<'a>) -> Self {
        let size = Size {
            width: raw.w as u32,
            height: raw.h as u32,
        };
        Self {
            flags: raw.flag,
            yc_p_edit: BorrowedMutFrame::from_raw(raw.yc_p_edit, size),
            yc_p_temp: BorrowedMutFrame::from_raw(raw.yc_p_temp, size),
            size,
            max_size: Size {
                width: raw.max_w as u32,
                height: raw.max_h as u32,
            },
            current_frame: raw.frame as usize,
            total_frames: raw.frame_n as usize,
            original_size: Size {
                width: raw.org_w as u32,
                height: raw.org_h as u32,
            },
            audio_buffer: AudioBuffer {
                data: std::slice::from_raw_parts_mut(
                    raw.audio_p,
                    (raw.audio_n * raw.audio_ch) as usize,
                ),
                total_samples: raw.audio_n as usize,
                channels: raw.audio_ch as usize,
            },
            editing: Editing::from_raw(raw.edit_p, api),
        }
    }
}

pub enum UpdateStatus {
    All,
    Track { index: usize },
    CheckboxOrButton { index: usize },
}

impl UpdateStatus {
    pub fn from_raw(raw: FilterUpdateStatus) -> Self {
        if raw == FilterUpdateStatus::ALL {
            Self::All
        } else if raw.contains(FilterUpdateStatus::TRACK) {
            Self::Track {
                index: raw.lower_bits() as usize,
            }
        } else if raw.contains(FilterUpdateStatus::CHECK) {
            Self::CheckboxOrButton {
                index: raw.lower_bits() as usize,
            }
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameInfo {
    pub frame_rate: usize,
    pub edit_flag: EditFlag,
    pub interlace: FrameInterlace,
}

pub mod prelude {
    pub use super::{
        api::Api, editing::Editing, window_message::WindowMessage, Control, FilterPlugin,
        FilterPluginFlag, FrameInfo, ProcInfo, Track, UpdateStatus,
    };
    pub use crate::{export_filter_plugin, AviUtlError, Result, Size};
}

pub trait FilterPlugin: Default {
    const NAME: &'static str;
    const INFORMATION: &'static str;
    const TRACKS: &'static [Track] = &[];
    const CONTROLS: &'static [Control] = &[];
    const WINDOW_SIZE: Size = Size::new();
    const FLAGS: FilterPluginFlag;
    fn process(&mut self, _proc_info: &mut ProcInfo) -> Result<()> {
        Ok(())
    }
    fn init(&mut self, _api: &Api) -> Result<()> {
        Ok(())
    }
    fn exit(&mut self, _api: &Api) -> Result<()> {
        Ok(())
    }
    fn update(&mut self, _status: UpdateStatus) -> Result<()> {
        Ok(())
    }
    fn handle_window(
        &mut self,
        _editing: Editing,
        _window: HWND,
        _dll: HINSTANCE,
        _message: WindowMessage,
    ) -> Result<bool> {
        Ok(false)
    }
    fn will_save(&mut self, _frames: RangeInclusive<usize>, _editing: Editing) -> Result<()> {
        Ok(())
    }
    fn did_save(&mut self, _editing: Editing) -> Result<()> {
        Ok(())
    }
    fn is_save_frame(
        &mut self,
        _editing: Editing,
        _asking: usize,
        _current: usize,
        _frame_info: FrameInfo,
    ) -> bool {
        true
    }
    fn load_project(&mut self, _editing: Editing, _load: &[u8]) -> Result<()> {
        Ok(())
    }
    fn save_project(&self, _editing: Editing, _save: &mut [u8]) -> Result<usize> {
        Ok(0)
    }
    fn modify_title(&self, _editing: Editing, _frame: usize) -> Result<Option<String>> {
        Ok(None)
    }
}

#[macro_export]
macro_rules! export_filter_plugin {
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
        struct PStrNames(&'static [*mut u8]);
        unsafe impl Send for PStrNames {}
        unsafe impl Sync for PStrNames {}
        static TRACK_NAMES: ::once_cell::sync::Lazy<PStrNames> =
            ::once_cell::sync::Lazy::new(|| {
                let names: Vec<_> = <$impl>::TRACKS
                    .into_iter()
                    .map(|track| {
                        ::std::ffi::CString::new(track.name).unwrap().into_raw() as *mut u8
                    })
                    .collect();
                PStrNames(names.leak())
            });
        static TRACK_DEFAULTS: ::once_cell::sync::Lazy<&'static [::std::os::raw::c_int]> =
            ::once_cell::sync::Lazy::new(|| {
                let defaults: Vec<_> = <$impl>::TRACKS
                    .into_iter()
                    .map(|track| track.default_value as ::std::os::raw::c_int)
                    .collect();
                defaults.leak()
            });
        static TRACK_MINS: ::once_cell::sync::Lazy<&'static [::std::os::raw::c_int]> =
            ::once_cell::sync::Lazy::new(|| {
                let mins: Vec<_> = <$impl>::TRACKS
                    .into_iter()
                    .map(|track| track.min_value as ::std::os::raw::c_int)
                    .collect();
                mins.leak()
            });
        static TRACK_MAXES: ::once_cell::sync::Lazy<&'static [::std::os::raw::c_int]> =
            ::once_cell::sync::Lazy::new(|| {
                let maxes: Vec<_> = <$impl>::TRACKS
                    .into_iter()
                    .map(|track| track.max_value as ::std::os::raw::c_int)
                    .collect();
                maxes.leak()
            });
        static CONTROL_NAMES: ::once_cell::sync::Lazy<PStrNames> =
            ::once_cell::sync::Lazy::new(|| {
                let names: Vec<_> = <$impl>::CONTROLS
                    .into_iter()
                    .map(|control| {
                        ::std::ffi::CString::new(control.name).unwrap().into_raw() as *mut u8
                    })
                    .collect();
                PStrNames(names.leak())
            });
        static CONTROL_DEFAULTS: ::once_cell::sync::Lazy<&'static [::std::os::raw::c_int]> =
            ::once_cell::sync::Lazy::new(|| {
                let defaults: Vec<_> = <$impl>::CONTROLS
                    .into_iter()
                    .map(|control| {
                        control.default_checked as ::std::os::raw::c_int
                            * if control.is_button { -1 } else { 1 }
                    })
                    .collect();
                defaults.leak()
            });
        #[no_mangle]
        unsafe extern "system" fn GetFilterPluginTable(
        ) -> *const ::aviutl_plugin_sys::filter::FilterPlugin {
            &*TABLE
        }
        static TABLE: ::once_cell::sync::Lazy<::aviutl_plugin_sys::filter::FilterPlugin> =
            ::once_cell::sync::Lazy::new(|| ::aviutl_plugin_sys::filter::FilterPlugin {
                flag: <$impl>::FLAGS,
                width: <$impl>::WINDOW_SIZE.width as ::std::os::raw::c_int,
                height: <$impl>::WINDOW_SIZE.height as ::std::os::raw::c_int,
                name: NAME.as_ptr() as _,
                track_n: <$impl>::TRACKS.len() as _,
                track_name: TRACK_NAMES.0.as_ptr(),
                track_default: TRACK_DEFAULTS.as_ptr(),
                track_s: TRACK_MINS.as_ptr(),
                track_e: TRACK_MAXES.as_ptr(),
                check_n: <$impl>::CONTROLS.len() as _,
                check_name: CONTROL_NAMES.0.as_ptr(),
                check_default: CONTROL_DEFAULTS.as_ptr(),
                func_proc,
                func_init,
                func_exit,
                func_update,
                func_window_proc,
                track: ::std::ptr::null(),
                check: ::std::ptr::null(),
                ex_data_ptr: ::std::ptr::null_mut(),
                ex_data_size: 0,
                information: INFORMATION.as_ptr() as _,
                func_save_start,
                func_save_end,
                ex_func: ::std::ptr::null(),
                window_handle: 0,
                dll_instance: 0,
                ex_data_def: ::std::ptr::null_mut(),
                func_is_saveframe,
                func_project_load,
                func_project_save,
                func_modify_title,
                dll_path: ::std::ptr::null_mut(),
                _reserve: [0; 2],
            });
        unsafe extern "system" fn func_proc(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            info: *const ::aviutl_plugin_sys::filter::FilterProcInfo,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let mut proc_info =
                unsafe { ::aviutl_plugin::filter::ProcInfo::from_raw(&*info, &api) };
            PLUGIN.lock().unwrap().process(&mut proc_info).is_ok() as _
        }
        unsafe extern "system" fn func_init(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            PLUGIN.lock().unwrap().init(&api).is_ok() as _
        }
        unsafe extern "system" fn func_exit(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            PLUGIN.lock().unwrap().exit(&api).is_ok() as _
        }
        unsafe extern "system" fn func_update(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            status: ::aviutl_plugin_sys::filter::FilterUpdateStatus,
        ) -> i32 {
            let update_status = ::aviutl_plugin::filter::UpdateStatus::from_raw(status);
            PLUGIN.lock().unwrap().update(update_status).is_ok() as _
        }
        unsafe extern "system" fn func_window_proc(
            window: isize,
            message: ::aviutl_plugin_sys::filter::WindowMessage,
            wparam: usize,
            lparam: isize,
            editing: *mut ::std::os::raw::c_void,
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            let window = ::windows::Win32::Foundation::HWND(window);
            let dll = ::windows::Win32::Foundation::HINSTANCE(TABLE.dll_instance);
            let message = ::aviutl_plugin::filter::window_message::WindowMessage::from(
                message,
                ::windows::Win32::Foundation::WPARAM(wparam),
                ::windows::Win32::Foundation::LPARAM(lparam),
            );
            PLUGIN
                .lock()
                .unwrap()
                .handle_window(editing, window, dll, message)
                .map_or(0, |bool| bool as i32)
        }
        unsafe extern "system" fn func_save_start(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            start: ::std::os::raw::c_int,
            end: ::std::os::raw::c_int,
            editing: *mut ::std::os::raw::c_void,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            PLUGIN
                .lock()
                .unwrap()
                .will_save((start as usize)..=(end as usize), editing)
                .is_ok() as _
        }
        unsafe extern "system" fn func_save_end(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            editing: *mut ::std::os::raw::c_void,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            PLUGIN.lock().unwrap().did_save(editing).is_ok() as _
        }
        unsafe extern "system" fn func_is_saveframe(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            editing: *mut ::std::os::raw::c_void,
            asking: ::std::os::raw::c_int,
            current: ::std::os::raw::c_int,
            frame_rate: ::std::os::raw::c_int,
            edit_flag: ::aviutl_plugin_sys::filter::EditFlag,
            interlace: ::aviutl_plugin_sys::filter::FrameInterlace,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            let info = ::aviutl_plugin::filter::FrameInfo {
                frame_rate: frame_rate as usize,
                edit_flag,
                interlace,
            };
            PLUGIN
                .lock()
                .unwrap()
                .is_save_frame(editing, asking as usize, current as usize, info) as _
        }
        unsafe extern "system" fn func_project_load(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            editing: *mut ::std::os::raw::c_void,
            load: *const ::std::os::raw::c_void,
            load_len: ::std::os::raw::c_int,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            let load = unsafe { ::std::slice::from_raw_parts(load.cast(), load_len as usize) };
            PLUGIN.lock().unwrap().load_project(editing, load).is_ok() as _
        }
        unsafe extern "system" fn func_project_save(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            editing: *mut ::std::os::raw::c_void,
            save: *mut ::std::os::raw::c_void,
            save_len: *mut ::std::os::raw::c_int,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            let save = unsafe { ::std::slice::from_raw_parts_mut(save.cast(), save_len as usize) };
            let res = PLUGIN.lock().unwrap().save_project(editing, save);
            if res.is_err() {
                unsafe {
                    *save_len = 0;
                }
                return 0;
            }
            unsafe {
                *save_len = res.unwrap() as ::std::os::raw::c_int;
            }
            1
        }
        unsafe extern "system" fn func_modify_title(
            _: *mut ::aviutl_plugin_sys::filter::FilterPlugin,
            editing: *mut ::std::os::raw::c_void,
            current_frame: ::std::os::raw::c_int,
            buf: *mut u8,
            buf_len: ::std::os::raw::c_int,
        ) -> i32 {
            let api = unsafe { ::aviutl_plugin::filter::api::Api::from_raw(&TABLE) };
            let editing =
                unsafe { ::aviutl_plugin::filter::editing::Editing::from_raw(editing, &api) };
            let mut buf = unsafe {
                // -1 due to terminating null byte.
                ::std::slice::from_raw_parts_mut(buf, buf_len as usize - 1)
            };
            let res = PLUGIN
                .lock()
                .unwrap()
                .modify_title(editing, current_frame as usize);
            if res.is_err() {
                return 0;
            }
            let new_title = res.unwrap();
            if let Some(new_title) = new_title {
                use ::std::io::Write;
                if buf.write(new_title.as_bytes()).is_err() {
                    return 0;
                }
            }
            1
        }
    };
}
