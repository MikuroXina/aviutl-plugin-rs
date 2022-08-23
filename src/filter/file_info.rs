use crate::{from_nullable_lpstr, FrameRate, Size};
use aviutl_plugin_sys::filter::FileInfo as RawFileInfo;

pub use aviutl_plugin_sys::filter::FileInfoFlag;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FileInfo {
    pub flags: FileInfoFlag,
    pub name: Option<String>,
    pub size: Size,
    pub frame_rate: FrameRate,
    pub total_frames: usize,
    pub decode_format: u32,
    pub decode_bits: usize,
    pub audio_rate: usize,
    pub audio_channels: usize,
    pub audio_samples: usize,
}

impl FileInfo {
    pub(crate) fn from_raw(raw: &RawFileInfo) -> Self {
        Self {
            flags: raw.flag,
            name: unsafe { from_nullable_lpstr(raw.name) },
            size: Size {
                width: raw.w as u32,
                height: raw.h as u32,
            },
            frame_rate: FrameRate {
                rate: raw.video_rate as u32,
                scale: raw.video_scale as u32,
            },
            total_frames: raw.frame_n as usize,
            decode_format: raw.video_decode_format,
            decode_bits: raw.video_decode_bit as usize,
            audio_rate: raw.audio_rate as usize,
            audio_channels: raw.audio_ch as usize,
            audio_samples: raw.audio_n as usize,
        }
    }
}
