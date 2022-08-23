//! Example from サンプルビデオフィルタ(フィルタプラグイン) for AviUtl ver0.99e or later by ＫＥＮくん.

use aviutl_plugin::{
    filter::{prelude::*, Frame},
    Rect,
};

#[derive(Debug, Default)]
struct VideoFilter;

impl FilterPlugin for VideoFilter {
    const NAME: &'static str = "サンプルフィルタ";
    const INFORMATION: &'static str = "サンプルフィルタ version 0.06 by ＫＥＮくん";

    const TRACKS: &'static [Track] = &[
        Track {
            name: "Y シフト",
            default_value: 0,
            min_value: -999,
            max_value: 999,
        },
        Track {
            name: "Cb シフト",
            default_value: 0,
            min_value: -999,
            max_value: 999,
        },
        Track {
            name: "Cr シフト",
            default_value: 0,
            min_value: -999,
            max_value: 999,
        },
    ];

    const CONTROLS: &'static [Control] = &[
        Control {
            name: "横幅を半分に縮小",
            default_checked: false,
            is_button: false,
        },
        Control {
            name: "縦横を半分に縮小",
            default_checked: false,
            is_button: false,
        },
    ];

    const FLAGS: FilterPluginFlag = FilterPluginFlag::EX_INFORMATION;

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        let tracks: [_; 3] =
            std::array::from_fn(|i| proc_info.editing.api().get_track_value(i).unwrap() as i16);
        for px in proc_info.yc_p_edit.image_mut() {
            px.y += tracks[0];
            px.cb += tracks[1];
            px.cr += tracks[2];
        }

        if proc_info.editing.api().get_check_value(0).unwrap() {
            for (temp, edit) in proc_info
                .yc_p_temp
                .lines_mut()
                .zip(proc_info.yc_p_edit.lines())
            {
                for (x, pair) in edit.chunks_exact(2).enumerate() {
                    temp[x] = pair[0] / 2 + pair[1] / 2;
                }
            }

            proc_info.size.width /= 2;
            std::mem::swap(&mut proc_info.yc_p_edit, &mut proc_info.yc_p_temp);
        }

        if proc_info.editing.api().get_check_value(1).unwrap() {
            proc_info.editing.resize(
                &mut proc_info.yc_p_temp,
                proc_info.size / 2,
                &proc_info.yc_p_edit,
                Rect {
                    size: proc_info.size,
                    ..Default::default()
                },
            );
            proc_info.editing.copy_from(
                &mut proc_info.yc_p_edit,
                proc_info.size / 4,
                &proc_info.yc_p_temp,
                Rect {
                    size: proc_info.size / 2,
                    ..Default::default()
                },
                2048,
            );
        }

        Ok(())
    }
}

export_filter_plugin!(VideoFilter);
