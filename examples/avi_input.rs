use aviutl_plugin::input::prelude::*;

struct AviHandle {}

impl InputHandle for AviHandle {}

#[derive(Debug, Default)]
struct AviInput;

impl InputPlugin for AviInput {
    type Handle = AviHandle;

    const NAME: &'static str = "AVI File Reader (example)";
    const INFORMATION: &'static str = "AVI File Reader version 0.03 By ＫＥＮくん";
    const FLAGS: PluginFlag = PluginFlag::AUDIO.union(PluginFlag::VIDEO);
    fn file_filters() -> FileFilters {
        let mut filter = FileFilters::new();
        filter.add_filter("AVI File (*.avi)", "*.avi");
        filter
    }
    fn open(&mut self, _path: std::borrow::Cow<'_, str>) -> Result<AviHandle> {
        todo!()
    }
    fn close(&mut self, _handle: AviHandle) -> Result<()> {
        todo!()
    }
}

export_input_plugin!(AviInput);
