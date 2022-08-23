use aviutl_plugin_sys::filter::FrameStatus as RawFrameStatus;

pub use aviutl_plugin_sys::filter::EditFlag;
pub use aviutl_plugin_sys::filter::FrameInterlace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FrameStatus {
    pub video_id: usize,
    pub audio_id: usize,
    pub interlace_mode: FrameInterlace,
    pub config_id: usize,
    pub vcm_id: usize,
    pub edit_flag: EditFlag,
}

impl FrameStatus {
    pub(crate) fn from_raw(raw: RawFrameStatus) -> Self {
        Self {
            video_id: raw.video as usize,
            audio_id: raw.audio as usize,
            interlace_mode: raw.inter,
            config_id: raw.config as usize,
            vcm_id: raw.vcm as usize,
            edit_flag: raw.edit_flag,
        }
    }

    pub(crate) fn into_raw(self) -> RawFrameStatus {
        RawFrameStatus {
            video: self.video_id as _,
            audio: self.audio_id as _,
            inter: self.interlace_mode,
            index24fps: 0,
            config: self.config_id as _,
            vcm: self.vcm_id as _,
            edit_flag: self.edit_flag,
            _reserve: [0; 9],
        }
    }
}
