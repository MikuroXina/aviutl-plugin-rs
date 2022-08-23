use super::{
    api::Api, file_info::FileInfo, frame_status::FrameStatus, sys_info::SysInfo, BorrowedMutFrame,
    FileId, Frame, VideoId,
};
use crate::{
    from_nullable_lpstr, into_win_str, AviUtlError, PixelFormat, PixelRgb, Point, Rect, Result,
    Size,
};
use std::{
    mem::MaybeUninit,
    ops::{Bound, RangeBounds, RangeInclusive},
    os::raw::c_void,
    ptr::null_mut,
};

pub use aviutl_plugin_sys::filter::EditOpenFlag as OpenFlag;
pub use aviutl_plugin_sys::filter::EditOutputFlag as OutputFlag;
pub use aviutl_plugin_sys::filter::FrameStatusType;
use windows::Win32::Graphics::Gdi::HFONT;

pub struct Editing<'a> {
    handle: *mut c_void,
    api: &'a Api<'a>,
}

impl<'a> Drop for Editing<'a> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<'a> Editing<'a> {
    /// Creates an editing file handle from the raw pointer.
    ///
    /// # Safety
    ///
    /// `handle` must be valid and received from AviUtl. Otherwise it will occur UB.
    pub unsafe fn from_raw(handle: *mut c_void, api: &'a Api<'a>) -> Self {
        Self { handle, api }
    }

    pub fn api(&self) -> &'a Api<'a> {
        self.api
    }

    pub fn open(&self, file_path: &str, flags: OpenFlag) -> Result<()> {
        let file_path = into_win_str(file_path);
        if unsafe { (self.api.exports.edit_open)(self.handle, file_path.as_ptr() as *mut _, flags) }
            == 0
        {
            Err(AviUtlError::File(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to open edit file",
            )))
        } else {
            Ok(())
        }
    }

    pub fn export(&self, file_path: &str, flags: OutputFlag, plugin_name: &str) -> Result<()> {
        let file_path = into_win_str(file_path);
        let plugin_name_cstr = into_win_str(plugin_name);
        if unsafe {
            (self.api.exports.edit_output)(
                self.handle,
                file_path.as_ptr() as *mut _,
                flags,
                plugin_name_cstr.as_ptr() as *mut _,
            )
        } == 0
        {
            Err(AviUtlError::Unsupported(format!(
                "export by plugin {}",
                plugin_name
            )))
        } else {
            Ok(())
        }
    }

    pub fn set_config(&self, profile: usize, profile_name: &str) -> Result<()> {
        let profile_name = into_win_str(profile_name);
        if unsafe {
            (self.api.exports.set_config)(
                self.handle,
                profile as _,
                profile_name.as_ptr() as *mut _,
            )
        } == 0
        {
            Err(AviUtlError::ConfigFailure("profile".into()))
        } else {
            Ok(())
        }
    }

    pub fn get_source_frame_from_avi(
        &self,
        frame: usize,
        offset: usize,
    ) -> Result<BorrowedMutFrame> {
        let ptr = unsafe {
            (self.api.exports.get_yc_p_source_cache)(self.handle, frame as _, offset as _)
        };
        if ptr.is_null() {
            Err(AviUtlError::FrameIndexOutOfRange(frame))
        } else {
            Ok(unsafe { BorrowedMutFrame::from_raw(ptr.cast(), self.frame_size()?) })
        }
    }

    pub fn get_source_frame(&self, frame: usize) -> Result<BorrowedMutFrame> {
        self.get_source_frame_from_avi(frame, 0)
    }

    pub fn get_source_dib_frame(&self, frame: usize, bytes_per_pixel: usize) -> Result<&[u8]> {
        let ptr = unsafe { (self.api.exports.get_pixel_p)(self.handle, frame as _) };
        if ptr.is_null() {
            Err(AviUtlError::FrameIndexOutOfRange(frame))
        } else {
            Ok(unsafe {
                std::slice::from_raw_parts(
                    ptr as *const _ as *const u8,
                    self.frame_size()?.area() * bytes_per_pixel,
                )
            })
        }
    }

    pub fn get_audio(&self, frame: usize, buf: &mut [u8]) -> Result<usize> {
        let expected_len =
            unsafe { (self.api.exports.get_audio)(self.handle, frame as _, null_mut()) as usize };
        if buf.len() < expected_len {
            return Err(AviUtlError::BufferLimitExceed);
        }
        Ok(unsafe {
            (self.api.exports.get_audio)(self.handle, frame as _, buf.as_mut_ptr().cast()) as usize
        })
    }

    pub fn is_editing(&self) -> bool {
        unsafe { (self.api.exports.is_editing)(self.handle) != 0 }
    }

    pub fn is_saving(&self) -> bool {
        unsafe { (self.api.exports.is_saving)(self.handle) != 0 }
    }

    pub fn current_frame(&self) -> usize {
        unsafe { (self.api.exports.get_frame)(self.handle) as usize }
    }

    pub fn total_frames(&self) -> usize {
        unsafe { (self.api.exports.get_frame_n)(self.handle) as usize }
    }

    pub fn frame_size(&self) -> Result<Size> {
        let (mut width, mut height) = (0, 0);
        if unsafe { (self.api.exports.get_frame_size)(self.handle, &mut width, &mut height) } == 0 {
            Err(AviUtlError::Unsupported("getting frame size".into()))
        } else {
            Ok(Size {
                width: width as u32,
                height: height as u32,
            })
        }
    }

    pub fn set_current_frame(&self, frame: usize) -> usize {
        unsafe { (self.api.exports.set_frame)(self.handle, frame as _) as usize }
    }

    pub fn set_total_frames(&self, frames: usize) -> usize {
        unsafe { (self.api.exports.set_frame_n)(self.handle, frames as _) as usize }
    }

    pub fn copy_video_audio(&self, dst: usize, src: usize) -> Result<()> {
        if unsafe { (self.api.exports.copy_frame)(self.handle, dst as _, src as _) } == 0 {
            Err(AviUtlError::FrameIndexOutOfRange(src))
        } else {
            Ok(())
        }
    }

    pub fn copy_video(&self, dst: usize, src: usize) -> Result<()> {
        if unsafe { (self.api.exports.copy_video)(self.handle, dst as _, src as _) } == 0 {
            Err(AviUtlError::FrameIndexOutOfRange(src))
        } else {
            Ok(())
        }
    }

    pub fn copy_audio(&self, dst: usize, src: usize) -> Result<()> {
        if unsafe { (self.api.exports.copy_audio)(self.handle, dst as _, src as _) } == 0 {
            Err(AviUtlError::FrameIndexOutOfRange(src))
        } else {
            Ok(())
        }
    }

    pub fn get_frame_status(&self, frame: usize) -> Result<FrameStatus> {
        let mut status = MaybeUninit::uninit();
        if unsafe {
            (self.api.exports.get_frame_status)(self.handle, frame as _, status.as_mut_ptr())
        } == 0
        {
            Err(AviUtlError::FrameIndexOutOfRange(frame))
        } else {
            Ok(FrameStatus::from_raw(unsafe { status.assume_init() }))
        }
    }

    pub fn set_frame_status(&self, frame: usize, status: FrameStatus) -> Result<()> {
        let raw = status.into_raw();
        if unsafe {
            (self.api.exports.set_frame_status)(self.handle, frame as _, &raw as *const _ as *mut _)
        } == 0
        {
            Err(AviUtlError::FrameIndexOutOfRange(frame))
        } else {
            Ok(())
        }
    }

    pub fn is_save_frame(&self, frame: usize) -> bool {
        unsafe { (self.api.exports.is_saveframe)(self.handle, frame as _) != 0 }
    }

    pub fn is_key_frame(&self, frame: usize) -> bool {
        unsafe { (self.api.exports.is_keyframe)(self.handle, frame as _) != 0 }
    }

    pub fn is_needed_recompression(&self, frame: usize) -> bool {
        unsafe { (self.api.exports.is_recompress)(self.handle, frame as _) != 0 }
    }

    pub fn get_file_info(&self) -> Result<FileInfo> {
        let mut raw = MaybeUninit::uninit();
        if unsafe { (self.api.exports.get_file_info)(self.handle, raw.as_mut_ptr()) } == 0 {
            Err(AviUtlError::Unsupported(
                "getting file info of editing".into(),
            ))
        } else {
            let raw = unsafe { raw.assume_init() };
            Ok(FileInfo::from_raw(&raw))
        }
    }

    pub fn get_config_name(&self, index: usize) -> Result<String> {
        let ptr = unsafe { (self.api.exports.get_config_name)(self.handle, index as _) };
        unsafe { from_nullable_lpstr(ptr) }
            .ok_or_else(|| AviUtlError::Unsupported("getting config name".into()))
    }

    pub fn get_filtered_dib_frame(&self, frame: usize, dib_buf: &mut [u8]) -> Result<Size> {
        let (mut width, mut height) = (0, 0);
        if unsafe {
            (self.api.exports.get_pixel_filtered)(
                self.handle,
                frame as _,
                null_mut(),
                &mut width,
                &mut height,
            )
        } == 0
        {
            return Err(AviUtlError::Unsupported(format!(
                "getting frame size at {}",
                frame
            )));
        }
        let size = Size {
            width: width as u32,
            height: height as u32,
        };
        let expected_len = size.area() * 3;
        if dib_buf.len() < expected_len {
            return Err(AviUtlError::BufferLimitExceed);
        }
        if unsafe {
            (self.api.exports.get_pixel_filtered)(
                self.handle,
                frame as _,
                dib_buf.as_mut_ptr().cast(),
                null_mut(),
                null_mut(),
            )
        } == 0
        {
            panic!("reading pixels should be succeed");
        }
        Ok(size)
    }

    pub fn get_filtered_audio(&self, frame: usize, wav_buf: &mut [u8]) -> Result<usize> {
        let expected_len = unsafe {
            (self.api.exports.get_audio_filtered)(self.handle, frame as _, null_mut()) as usize
        };
        if wav_buf.len() < expected_len {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(unsafe {
                (self.api.exports.get_audio_filtered)(
                    self.handle,
                    frame as _,
                    wav_buf.as_mut_ptr().cast(),
                )
            } as usize)
        }
    }

    pub fn selected_frame_range(&self) -> Result<RangeInclusive<usize>> {
        let (mut start, mut end) = (0, 0);
        if unsafe { (self.api.exports.get_select_frame)(self.handle, &mut start, &mut end) } == 0 {
            Err(AviUtlError::Unsupported("getting selected range".into()))
        } else {
            Ok((start as usize)..=(end as usize))
        }
    }

    pub fn set_selected_frame_range<R: RangeBounds<usize>>(&self, range: R) -> Result<()> {
        let len = self.total_frames();
        let start = match range.start_bound() {
            Bound::Included(&idx) => idx,
            Bound::Excluded(&idx) if idx == len => len,
            Bound::Excluded(&idx) => idx + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&idx) => idx,
            Bound::Excluded(&idx) => idx.saturating_sub(1),
            Bound::Unbounded => len,
        };

        self.set_selected_frame_range_inner(start, end)
    }

    fn set_selected_frame_range_inner(&self, start: usize, end: usize) -> Result<()> {
        if unsafe { (self.api.exports.set_select_frame)(self.handle, start as _, end as _) } == 0 {
            Err(AviUtlError::Unsupported("setting selected range".into()))
        } else {
            Ok(())
        }
    }

    pub fn get_source_video_id(&self, frame: usize) -> Result<(FileId, VideoId)> {
        let (mut file_id, mut video_id) = (0, 0);
        if unsafe {
            (self.api.exports.get_source_video_number)(
                self.handle,
                frame as _,
                &mut file_id,
                &mut video_id,
            )
        } == 0
        {
            Err(AviUtlError::Unsupported("getting source id".into()))
        } else {
            Ok((FileId(file_id as u32), VideoId(video_id as u32)))
        }
    }

    pub fn get_sys_info(&self) -> Result<SysInfo> {
        let mut raw = MaybeUninit::uninit();
        if unsafe { (self.api.exports.get_sys_info)(self.handle, raw.as_mut_ptr()) } == 0 {
            Err(AviUtlError::Unsupported(
                "getting system information".into(),
            ))
        } else {
            let raw = unsafe { raw.assume_init() };
            Ok(SysInfo::from_raw(&raw))
        }
    }

    pub fn get_filtering_audio(&self, frame: usize, buf: &mut [u8]) -> Result<usize> {
        let expected_len = unsafe {
            (self.api.exports.get_audio_filtering)(
                self.api.filter,
                self.handle,
                frame as _,
                null_mut(),
            )
        } as usize;
        if buf.len() < expected_len {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(unsafe {
                (self.api.exports.get_audio_filtering)(
                    self.api.filter,
                    self.handle,
                    frame as _,
                    buf.as_mut_ptr().cast(),
                )
            } as usize)
        }
    }

    pub fn get_displaying(&self, format: PixelFormat) -> Result<&[u8]> {
        let ptr = unsafe { (self.api.exports.get_disp_pixel_p)(self.handle, format.into_raw()) };
        if ptr.is_null() {
            Err(AviUtlError::Unsupported("getting displaying frame".into()))
        } else {
            Ok(unsafe {
                std::slice::from_raw_parts(
                    ptr as *const _ as *const u8,
                    format.bytes_per_pixel() * self.frame_size()?.area(),
                )
            })
        }
    }

    pub fn get_filtered_dib_frame_with(
        &self,
        frame: usize,
        dib: &mut [u8],
        format: Option<PixelFormat>,
    ) -> Result<Size> {
        let format = format.unwrap_or_default();
        let (mut width, mut height) = (0, 0);
        if unsafe {
            (self.api.exports.get_pixel_filtered_ex)(
                self.handle,
                frame as _,
                null_mut(),
                &mut width,
                &mut height,
                format.into_raw(),
            )
        } == 0
        {
            return Err(AviUtlError::Unsupported(format!(
                "rendering with format {:?}",
                format
            )));
        }
        let size = Size {
            width: width as u32,
            height: height as u32,
        };
        let expected_len = size.area() * format.bytes_per_pixel();
        if dib.len() < expected_len {
            return Err(AviUtlError::BufferLimitExceed);
        }
        if unsafe {
            (self.api.exports.get_pixel_filtered_ex)(
                self.handle,
                frame as _,
                dib.as_mut_ptr().cast(),
                null_mut(),
                null_mut(),
                format.into_raw(),
            )
        } == 0
        {
            panic!("render should succeed");
        }
        Ok(size)
    }

    pub fn get_yc_filtering(&self, frame: usize) -> Result<BorrowedMutFrame> {
        let (mut width, mut height) = (0, 0);
        let ptr = unsafe {
            (self.api.exports.get_yc_p_filtering_cache_ex)(
                self.api.filter,
                self.handle,
                frame as _,
                &mut width,
                &mut height,
            )
        };
        if ptr.is_null() {
            Err(AviUtlError::FrameIndexOutOfRange(frame))
        } else {
            Ok(unsafe {
                BorrowedMutFrame::from_raw(
                    ptr,
                    Size {
                        width: width as u32,
                        height: height as u32,
                    },
                )
            })
        }
    }

    pub fn get_frame_status_table(&self, status: FrameStatusType) -> *const u8 {
        unsafe { (self.api.exports.get_frame_status_table)(self.handle, status) }
    }

    pub fn set_undo(&self) -> Result<()> {
        if unsafe { (self.api.exports.set_undo)(self.handle) } == 0 {
            Err(AviUtlError::Unsupported("undo".into()))
        } else {
            Ok(())
        }
    }
}

impl Editing<'_> {
    pub fn load_bmp(&self, frame: &mut impl Frame, file_name: &str) -> Result<Size> {
        let file_name_cstr = into_win_str(file_name);
        let (mut width, mut height) = (0, 0);
        if unsafe {
            (self.api().exports.load_image)(
                frame.image_mut().as_mut_ptr(),
                file_name_cstr.as_ptr() as *mut _,
                &mut width,
                &mut height,
                0,
            )
        } == 0
        {
            Err(AviUtlError::Load(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to open file: {}", file_name),
            )))
        } else {
            Ok(Size {
                width: width as _,
                height: height as _,
            })
        }
    }

    pub fn resize(&self, frame: &mut impl Frame, target: Size, source: &impl Frame, clop: Rect) {
        unsafe {
            (self.api().exports.resize_yc)(
                frame.image_mut().as_mut_ptr(),
                target.width as _,
                target.height as _,
                source.image().as_ptr(),
                clop.point.x as _,
                clop.point.y as _,
                clop.size.width as _,
                clop.size.height as _,
            );
        }
    }

    pub fn copy_from(
        &self,
        frame: &mut impl Frame,
        target: Size,
        source: &impl Frame,
        clop: Rect,
        opacity: u16,
    ) {
        unsafe {
            (self.api().exports.copy_yc)(
                frame.image_mut().as_mut_ptr(),
                target.width as _,
                target.height as _,
                source.image().as_ptr(),
                clop.point.x as _,
                clop.point.y as _,
                clop.size.width as _,
                clop.size.height as _,
                opacity.clamp(0, 4096) as _,
            );
        }
    }

    pub fn draw_text(
        &self,
        frame: &mut impl Frame,
        pos: Point,
        text: &str,
        color: PixelRgb,
        opacity: u16,
        font: Option<HFONT>,
    ) -> Size {
        let text_cstr = into_win_str(text);
        let (mut width, mut height) = (0, 0);
        unsafe {
            (self.api().exports.draw_text)(
                frame.image_mut().as_mut_ptr(),
                pos.x as _,
                pos.y as _,
                text_cstr.as_ptr() as *mut _,
                color.r as _,
                color.g as _,
                color.b as _,
                opacity.clamp(0, 4096) as _,
                font.unwrap_or_default().0,
                &mut width,
                &mut height,
            );
        }
        Size {
            width: width as u32,
            height: height as u32,
        }
    }
}
