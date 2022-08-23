//! Example from サンプルオーディオフィルタ(フィルタプラグイン) for AviUtl ver0.96c or later by ＫＥＮくん.

use aviutl_plugin::filter::prelude::*;

#[derive(Debug, Default)]
struct AudioFilter;

impl FilterPlugin for AudioFilter {
    const NAME: &'static str = "音量の調節";
    const INFORMATION: &'static str = "音量の調節 version 0.03 by ＫＥＮくん";

    const TRACKS: &'static [Track] = &[Track {
        name: "レベル",
        default_value: 256,
        min_value: 0,
        max_value: 256,
    }];

    const FLAGS: FilterPluginFlag = FilterPluginFlag::PRIORITY_HIGHEST
        .union(FilterPluginFlag::AUDIO_FILTER)
        .union(FilterPluginFlag::EX_INFORMATION);

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        let volume = proc_info.editing.api().get_track_value(0).unwrap();
        for ch in 0..proc_info.audio_buffer.channels() {
            for sample in proc_info.audio_buffer.samples_by_channel(ch) {
                *sample = (*sample as f64 * volume as f64 / 256.0) as i16;
            }
        }
        Ok(())
    }
}

export_filter_plugin!(AudioFilter);
