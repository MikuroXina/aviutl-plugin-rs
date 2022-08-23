//! Example from マルチスレッド対応サンプルフィルタ(フィルタプラグイン) for AviUtl ver0.99a or later by ＫＥＮくん.

use aviutl_plugin::{
    filter::{prelude::*, Frame},
    PixelYc,
};

#[derive(Debug, Default)]
struct MultiThreadFilter {}

impl FilterPlugin for MultiThreadFilter {
    const NAME: &'static str = "マルチスレッドサンプルフィルタ";
    const INFORMATION: &'static str = "マルチスレッドサンプルフィルタ version 0.01 by ＫＥＮくん";

    const TRACKS: &'static [Track] = &[
        Track {
            name: "Y シフト",
            default_value: 512,
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

    const WINDOW_SIZE: Size = Size::new();

    const FLAGS: FilterPluginFlag = FilterPluginFlag::EX_INFORMATION;

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        let size = proc_info.size;
        let tracks: [_; 3] =
            std::array::from_fn(|i| proc_info.editing.api().get_track_value(i).unwrap());
        let edit = proc_info.yc_p_edit.image();
        proc_info.editing.api().exec_multi_thread_func(&|id, num| {
            let y_start = size.height as usize * id / num;
            // Safety: Referencing region of `edit` will not overlap.
            for px in unsafe {
                std::slice::from_raw_parts_mut(
                    edit.as_ptr().add(y_start) as *mut PixelYc,
                    size.width as usize,
                )
            } {
                px.y += tracks[0] as i16;
                px.cb += tracks[1] as i16;
                px.cr += tracks[2] as i16;
            }
        })?;
        Ok(())
    }
}

export_filter_plugin!(MultiThreadFilter);
