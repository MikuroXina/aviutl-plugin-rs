//! Example from サンプル表示プラグイン(フィルタプラグイン) for AviUtl ver0.98c以降 by ＫＥＮくん.

use aviutl_plugin::filter::prelude::*;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, RECT},
        Graphics::Gdi::{
            FillRect, GetDC, ReleaseDC, SetStretchBltMode, StretchDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBRUSH, HDC, SRCCOPY, STRETCH_DELETESCANS,
        },
        UI::WindowsAndMessaging::{
            CreateDialogParamA, DestroyWindow, GetClientRect, GetWindowRect, IsWindowVisible,
            SetWindowPos, ShowWindow, COLOR_INACTIVEBORDER, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SW_HIDE, SW_SHOW, WM_PAINT,
        },
    },
};

const WINDOW_WIDTH: u32 = 200 + 6;
const WINDOW_HEIGHT: u32 = 150 + 21;

#[derive(Debug, Default)]
struct DispFilter {
    this_window: HWND,
    child_window: HWND,
    buffers: Option<[Vec<u8>; 2]>,
    bitmap: BITMAPINFO,
}

impl DispFilter {
    fn show(&mut self, editing: Editing) -> Result<()> {
        unsafe {
            ShowWindow(self.this_window, SW_SHOW);
        }
        let sys_info = editing.get_sys_info()?;
        let size = sys_info.vram_size.area() * 3;
        self.buffers = Some([vec![0; size], vec![0; size]]);
        Ok(())
    }

    fn hide(&mut self) {
        unsafe {
            ShowWindow(self.this_window, SW_HIDE);
        }
        self.buffers = None;
    }

    fn display(&self, editing: Editing) -> Result<()> {
        if !unsafe { IsWindowVisible(self.child_window).as_bool() } {
            return Ok(());
        }

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

        let (rect, this_ctx, child_ctx) = unsafe {
            let mut rect = RECT::default();
            GetClientRect(self.this_window, &mut rect);
            let this_ctx = GetDC(self.this_window);
            let child_ctx = GetDC(self.child_window);
            (
                rect,
                ContextGuard(this_ctx, self.this_window),
                ContextGuard(child_ctx, self.child_window),
            )
        };
        let is_editing = editing.is_editing();
        if !is_editing || this_ctx.is_invalid() || child_ctx.is_invalid() {
            unsafe {
                FillRect(*this_ctx, &rect, HBRUSH(COLOR_INACTIVEBORDER.0 as _));
                FillRect(*child_ctx, &rect, HBRUSH(COLOR_INACTIVEBORDER.0 as _));
            }
        } else if let Some([buf0, buf1]) = &self.buffers {
            let header = self.bitmap.bmiHeader;
            unsafe {
                SetStretchBltMode(*this_ctx, STRETCH_DELETESCANS);
                SetStretchBltMode(*child_ctx, STRETCH_DELETESCANS);
                StretchDIBits(
                    *this_ctx,
                    0,
                    0,
                    rect.right,
                    rect.bottom,
                    0,
                    0,
                    header.biWidth,
                    header.biHeight,
                    buf0.as_ptr().cast(),
                    &self.bitmap,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                StretchDIBits(
                    *child_ctx,
                    0,
                    0,
                    rect.right,
                    rect.bottom,
                    0,
                    0,
                    header.biWidth,
                    header.biHeight,
                    buf1.as_ptr().cast(),
                    &self.bitmap,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
        }

        Ok(())
    }
}

impl FilterPlugin for DispFilter {
    const NAME: &'static str = "前後表示";
    const INFORMATION: &'static str = "前後表示 version 0.01 by ＫＥＮくん";

    const WINDOW_SIZE: Size = Size {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    };

    const FLAGS: FilterPluginFlag = FilterPluginFlag::ALWAYS_ACTIVE
        .union(FilterPluginFlag::MAIN_MESSAGE)
        .union(FilterPluginFlag::WINDOW_SIZE)
        .union(FilterPluginFlag::DISP_FILTER)
        .union(FilterPluginFlag::EX_INFORMATION);

    fn process(&mut self, proc_info: &mut ProcInfo) -> Result<()> {
        if let Some([buf0, buf1]) = &mut self.buffers {
            self.bitmap = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as _,
                    biWidth: proc_info.original_size.width as _,
                    biHeight: proc_info.original_size.height as _,
                    biPlanes: 1,
                    biBitCount: 24,
                    biCompression: BI_RGB as _,
                    ..Default::default()
                },
                ..Default::default()
            };
            let current = proc_info.editing.current_frame();
            *buf0 = proc_info
                .editing
                .get_source_dib_frame(current.saturating_sub(1), 3)?
                .to_vec();
            *buf1 = proc_info
                .editing
                .get_source_dib_frame(current.saturating_add(1), 3)?
                .to_vec();
        }
        Ok(())
    }

    fn init(&mut self, api: &Api) -> Result<()> {
        let plugin_window = api.plugin_window();
        self.child_window = unsafe {
            CreateDialogParamA(
                api.dll_instance(),
                PCSTR("DLG\0".as_ptr()),
                plugin_window,
                None,
                LPARAM::default(),
            )
        };
        self.this_window = plugin_window;
        unsafe {
            SetWindowPos(
                plugin_window,
                HWND::default(),
                0,
                0,
                WINDOW_WIDTH as _,
                WINDOW_HEIGHT as _,
                SWP_NOMOVE | SWP_NOZORDER,
            );
        }
        Ok(())
    }
    fn exit(&mut self, _api: &Api) -> Result<()> {
        unsafe {
            DestroyWindow(self.child_window);
        }
        Ok(())
    }

    fn handle_window(
        &mut self,
        editing: Editing,
        _window: HWND,
        _dll: HINSTANCE,
        message: WindowMessage,
    ) -> Result<bool> {
        match message {
            WindowMessage::System {
                original: WM_PAINT, ..
            }
            | WindowMessage::ChangeEdit => {
                self.display(editing)?;
            }
            WindowMessage::ChangeWindow => {
                if editing.api().is_displaying_window() {
                    self.show(editing)?;
                    return Ok(true);
                }
                self.hide();
            }
            WindowMessage::MainMoveSize { window } => {
                let mut rect = RECT::default();
                unsafe {
                    GetWindowRect(window, &mut rect);
                    SetWindowPos(
                        self.this_window,
                        HWND::default(),
                        rect.left - WINDOW_WIDTH as i32,
                        rect.top,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                    SetWindowPos(
                        self.child_window,
                        HWND::default(),
                        rect.right,
                        rect.top,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
            _ => {}
        }
        Ok(false)
    }
}

export_filter_plugin!(DispFilter);
