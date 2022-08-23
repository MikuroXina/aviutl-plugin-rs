use super::{api::Api, file_info::FileInfo, Frame};
use crate::{AviUtlError, Result};
use aviutl_plugin_sys::filter::{AviFileHandle, FileOpenFlag};
use std::mem::MaybeUninit;

pub struct AviFile<'a> {
    api: &'a Api<'a>,
    handle: AviFileHandle,
    file_info: FileInfo,
}

impl Drop for AviFile<'_> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<'a> AviFile<'a> {
    pub fn new(api: &'a Api<'a>, file_name: &str, open_flag: FileOpenFlag) -> Result<Self> {
        let (file_name, _, had_error) = encoding_rs::SHIFT_JIS.encode(file_name);
        if had_error {
            return Err(AviUtlError::Load(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to encode file name",
            )));
        }
        let mut file_name_bytes = file_name.into_owned();
        file_name_bytes.push(0);
        let mut info = MaybeUninit::uninit();
        let handle = unsafe {
            (api.exports.avi_file_open)(file_name_bytes.as_mut_ptr(), info.as_mut_ptr(), open_flag)
        };
        if handle.is_null() {
            return Err(AviUtlError::Load(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to open file",
            )));
        }
        let info = unsafe { info.assume_init() };
        Ok(Self {
            api,
            handle,
            file_info: FileInfo::from_raw(&info),
        })
    }

    pub fn file_info(&self) -> &FileInfo {
        &self.file_info
    }

    pub fn read_video(&mut self, target: &mut impl Frame, frame: usize) -> Result<()> {
        if unsafe {
            (self.api.exports.avi_file_read_video)(
                self.handle,
                target.image_mut().as_mut_ptr(),
                frame as _,
            )
        } == 0
        {
            Err(AviUtlError::BufferLimitExceed)
        } else {
            Ok(())
        }
    }

    pub fn read_audio(&mut self, target: &mut [u8], frame: usize) -> usize {
        unsafe {
            (self.api.exports.avi_file_read_audio)(
                self.handle,
                target.as_mut_ptr().cast(),
                frame as _,
            ) as usize
        }
    }

    pub fn get_video_dib(&mut self, frame: usize, bytes_per_pixel: usize) -> Result<&[u8]> {
        let ptr = unsafe {
            (self.api.exports.avi_file_get_video_pixel_p)(self.handle, frame as _) as *mut u8
        };
        if ptr.is_null() {
            return Err(AviUtlError::Unsupported(
                "getting video frame by dib format".into(),
            ));
        }
        let size = self.file_info.size;
        unsafe {
            Ok(std::slice::from_raw_parts(
                ptr,
                size.width as usize * size.height as usize * bytes_per_pixel,
            ))
        }
    }

    pub fn read_audio_sample(&mut self, frame: usize, buf: &mut [u8]) -> usize {
        unsafe {
            (self.api.exports.avi_file_read_audio_sample)(
                self.handle,
                frame as _,
                buf.len() as _,
                buf.as_mut_ptr().cast(),
            ) as usize
        }
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) -> u32 {
        unsafe {
            (self.api.exports.avi_file_set_audio_sample_rate)(
                self.handle,
                sample_rate as _,
                self.file_info.audio_channels as _,
            ) as u32
        }
    }
}
