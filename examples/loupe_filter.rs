//! Example from サンプル表示プラグイン(フィルタプラグイン) for AviUtl ver0.98c or later by ＫＥＮくん.

use aviutl_plugin::{filter::prelude::*, Point};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        FillRect, GetDC, ReleaseDC, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER,
        BI_RGB, DIB_RGB_COLORS, HBRUSH, HDC, SRCCOPY, STRETCH_DELETESCANS,
    },
    UI::WindowsAndMessaging::{GetClientRect, COLOR_INACTIVEBORDER, WM_PAINT},
};

const LOUPE_WIDTH: u32 = 32;
const LOUPE_HEIGHT: u32 = 32;
const WINDOW_WIDTH: u32 = LOUPE_WIDTH * 7 + 6;
const WINDOW_HEIGHT: u32 = LOUPE_HEIGHT * 7 + 21;

#[derive(Debug, Default)]
struct LoupeFilter {
    buf: Option<Vec<u8>>,
    bitmap: BITMAPINFO,
    zoom_pos: Point,
}

impl LoupeFilter {
    fn show(&mut self, editing: Editing) -> Result<()> {
        let sys_info = editing.get_sys_info()?;
        let size = sys_info.vram_size.area() * 3;
        self.buf = Some(vec![0; size]);
        Ok(())
    }

    fn hide(&mut self) {
        self.buf = None;
    }

    fn display(&mut self, editing: &Editing, zoom_point: Option<Point>) {
        struct ContextGuard(HDC, HWND);
        impl std::ops::Deref for ContextGuard {
            type Target = HDC;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl Drop for ContextGuard {
            fn drop(&mut self) {
                unsafe {
                    ReleaseDC(self.1, self.0);
                }
            }
        }

        let window = editing.api().plugin_window();
        let (rect, ctx) = unsafe {
            let mut rect = RECT::default();
            GetClientRect(window, &mut rect);
            let ctx = GetDC(window);
            (rect, ContextGuard(ctx, window))
        };
        let is_editing = editing.is_editing();

        if !is_editing || self.buf.is_none() {
            unsafe {
                FillRect(*ctx, &rect, HBRUSH(COLOR_INACTIVEBORDER.0 as _));
            }
            return;
        }
        if let Some(p) = zoom_point {
            let x = (p.x - LOUPE_WIDTH as i32 / 2)
                .clamp(0, self.bitmap.bmiHeader.biWidth as i32 - LOUPE_WIDTH as i32);
            let y = (p.y - LOUPE_HEIGHT as i32 / 2).clamp(
                0,
                self.bitmap.bmiHeader.biHeight as i32 - LOUPE_HEIGHT as i32,
            );
            self.zoom_pos = Point { x, y };
        }
        unsafe {
            SetStretchBltMode(*ctx, STRETCH_DELETESCANS);
            StretchDIBits(
                *ctx,
                0,
                0,
                rect.right,
                rect.bottom,
                self.zoom_pos.x as _,
                (self.bitmap.bmiHeader.biHeight - self.zoom_pos.y as i32 - LOUPE_HEIGHT as i32)
                    as _,
                LOUPE_WIDTH as _,
                LOUPE_HEIGHT as _,
                self.buf.as_ref().unwrap().as_ptr().cast(),
                &self.bitmap,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }
}

impl FilterPlugin for LoupeFilter {
    const NAME: &'static str = "簡易ルーペ";
    const INFORMATION: &'static str = "簡易ルーペ version 0.01 by ＫＥＮくん";

    const FLAGS: FilterPluginFlag = FilterPluginFlag::ALWAYS_ACTIVE
        .union(FilterPluginFlag::MAIN_MESSAGE)
        .union(FilterPluginFlag::WINDOW_SIZE)
        .union(FilterPluginFlag::DISP_FILTER)
        .union(FilterPluginFlag::EX_INFORMATION);

    const WINDOW_SIZE: Size = Size {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    };

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        if let Some(buf) = self.buf.as_mut() {
            let size = proc_info
                .editing
                .get_filtered_dib_frame(proc_info.current_frame, buf)?;
            self.bitmap = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as _,
                    biWidth: size.width as _,
                    biHeight: size.height as _,
                    biPlanes: 1,
                    biBitCount: 24,
                    biCompression: BI_RGB as _,
                    ..Default::default()
                },
                ..Default::default()
            };
            self.display(&proc_info.editing, None);
        }
        Ok(())
    }

    fn handle_window(
        &mut self,
        editing: Editing,
        _window: windows::Win32::Foundation::HWND,
        _dll: windows::Win32::Foundation::HINSTANCE,
        message: WindowMessage,
    ) -> Result<bool> {
        match message {
            WindowMessage::System {
                original: WM_PAINT, ..
            }
            | WindowMessage::ChangeEdit => {
                self.display(&editing, None);
            }
            WindowMessage::ChangeWindow => {
                if editing.api().is_displaying_window() {
                    self.show(editing)?;
                    return Ok(true);
                }
                self.hide();
            }
            WindowMessage::MainMouseMove { coordinate } => {
                self.display(&editing, Some(coordinate));
            }
            _ => {}
        }
        Ok(false)
    }
}

export_filter_plugin!(LoupeFilter);
