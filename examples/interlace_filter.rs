//! Example from サンプルインターレース解除プラグイン for AviUtl ver0.98 or later by ＫＥＮくん.

use aviutl_plugin::filter::{prelude::*, Frame};

#[derive(Debug, Default)]
struct InterlacePlugin;

impl FilterPlugin for InterlacePlugin {
    const NAME: &'static str = "サンプル解除";
    const INFORMATION: &'static str = "サンプルインターレース解除 version 0.01 by ＫＥＮくん";

    const FLAGS: FilterPluginFlag = FilterPluginFlag::INTERLACE_FILTER
        .union(FilterPluginFlag::EX_INFORMATION)
        .union(FilterPluginFlag::NO_CONFIG);

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        let width = proc_info.size.width as usize;
        let max_width = proc_info.max_size.width as usize;
        let mut frame = proc_info
            .editing
            .get_source_frame_from_avi(proc_info.current_frame, 0)?;

        // Odd de-interlace
        for y in (0..proc_info.editing.frame_size()?.height).filter(|y| y % 2 == 1) {
            let odd_index = (y - (y & 1)) as usize;
            let src_start = odd_index * max_width;
            let dst_start = y as usize * max_width;
            let (src, dst) = frame.image_mut()[src_start..dst_start + width].split_at_mut(width);
            dst.copy_from_slice(src);
        }
        Ok(())
    }

    fn is_save_frame(
        &mut self,
        _editing: Editing,
        asking: usize,
        _current: usize,
        frame_info: FrameInfo,
    ) -> bool {
        let fps = frame_info.frame_rate;
        asking * fps / 30 == (asking + 1) * fps / 30
    }
}

export_filter_plugin!(InterlacePlugin);
