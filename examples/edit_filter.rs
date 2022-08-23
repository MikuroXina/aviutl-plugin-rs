//! Example from サンプル編集プラグイン(フィルタプラグイン) for AviUtl ver0.99i or later by ＫＥＮくん.

use aviutl_plugin::filter::{file_info::FileInfoFlag, prelude::*, EditFlag};
use std::{ffi::CString, mem::MaybeUninit};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HINSTANCE, HWND},
        Graphics::Gdi::{
            FillRect, GetDC, ReleaseDC, SetStretchBltMode, StretchDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBRUSH, HDC, SRCCOPY, STRETCH_DELETESCANS,
        },
        UI::{
            Controls::SetScrollInfo,
            WindowsAndMessaging::{
                DrawMenuBar, GetClientRect, GetScrollInfo, LoadMenuA, MessageBoxA, PeekMessageA,
                SetMenu, SetWindowTextA, COLOR_INACTIVEBORDER, MB_OK, PM_NOREMOVE, SB_HORZ,
                SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBTRACK, SCROLLINFO,
                SIF_DISABLENOSCROLL, SIF_PAGE, SIF_POS, SIF_RANGE, WM_HSCROLL, WM_KEYDOWN,
                WM_MOUSEMOVE, WM_PAINT,
            },
        },
    },
};

const WINDOW_WIDTH: u32 = 320;
const WINDOW_HEIGHT: u32 = 240;

const VK_LEFT: u32 = 0x25;
const VK_RIGHT: u32 = 0x27;

mod command_index {
    pub const VIDEO_COPY: u32 = 100;
    pub const AUDIO_COPY: u32 = 101;
    pub const COPY: u32 = 102;
    pub const PASTE: u32 = 103;
    pub const DELETE: u32 = 104;
    pub const INSERT: u32 = 105;
    pub const FILE_INFO: u32 = 106;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum CopyMode {
    Video,
    Audio,
    #[default]
    All,
}

#[derive(Debug, Default)]
struct EditFilter {
    frame: usize,
    total_frame: usize,
    size: Size,
    copying_frame: usize,
    copy_mode: CopyMode,
    example_data: [u8; 3],
}

impl EditFilter {
    fn add_frame(&mut self, amount: usize) {
        self.frame = self.frame.saturating_add(amount);
        if self.total_frame <= self.frame {
            self.frame = self.total_frame - 1;
        }
    }

    fn sub_frame(&mut self, amount: usize) {
        self.frame = self.frame.saturating_sub(amount);
    }

    fn disp(&self, window: HWND, editing: Editing) -> Result<()> {
        if !editing.api().is_displaying_window() {
            return Ok(());
        }
        self.show_frame(window, &editing)?;
        self.show_title_bar(window, &editing)
    }

    fn show_frame(&self, window: HWND, editing: &Editing) -> Result<()> {
        let mut rect = MaybeUninit::uninit();
        let rect = unsafe {
            GetClientRect(window, rect.as_mut_ptr()).ok()?;
            rect.assume_init()
        };

        struct HdcGuard(HDC, HWND);
        impl Drop for HdcGuard {
            fn drop(&mut self) {
                unsafe {
                    ReleaseDC(self.1, self.0);
                }
            }
        }
        impl From<&'_ HdcGuard> for HDC {
            fn from(guard: &'_ HdcGuard) -> Self {
                guard.0
            }
        }
        let hdc = HdcGuard(unsafe { GetDC(window) }, window);
        unsafe {
            SetStretchBltMode(&hdc, STRETCH_DELETESCANS);
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as _,
                biWidth: self.size.width as _,
                biHeight: self.size.height as _,
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB as _,
                ..Default::default()
            },
            ..Default::default()
        };
        if editing.is_editing() && self.total_frame != 0 {
            unsafe {
                StretchDIBits(
                    &hdc,
                    0,
                    0,
                    rect.right,
                    rect.bottom,
                    0,
                    0,
                    self.size.width as _,
                    self.size.height as _,
                    editing.get_source_dib_frame(self.frame, 3)?.as_ptr().cast(),
                    &bmi,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
        } else {
            unsafe {
                FillRect(&hdc, &rect, HBRUSH(COLOR_INACTIVEBORDER.0 as _));
            }
        }
        Ok(())
    }

    fn show_title_bar(&self, window: HWND, editing: &Editing) -> Result<()> {
        if !editing.is_editing() || self.total_frame == 0 {
            let cstr = CString::new(Self::NAME).unwrap();
            unsafe {
                SetWindowTextA(window, PCSTR(cstr.as_ptr().cast())).ok()?;
            }
            return Ok(());
        }

        const INTERLACE_TEXTS: &[&str] = &["", "反転", "奇数", "偶数", "二重化", "自動"];
        let status = editing.get_frame_status(self.frame)?;
        let mut text = format!(
            "{}  [{}/{}]  {} {}",
            Self::NAME,
            self.frame + 1,
            self.total_frame,
            editing.get_config_name(status.config_id)?,
            INTERLACE_TEXTS[status.interlace_mode as usize]
        );
        if editing.is_key_frame(self.frame) {
            text.push('*');
        }
        if editing.is_needed_recompression(self.frame) {
            text.push('!');
        }
        if !editing.is_save_frame(self.frame) {
            text.push('X');
        }
        if status.edit_flag.contains(EditFlag::KEYFRAME) {
            text.push('K');
        }
        if status.edit_flag.contains(EditFlag::MARK_FRAME) {
            text.push('M');
        }
        if status.edit_flag.contains(EditFlag::DEL_FRAME) {
            text.push('D');
        }
        if status.edit_flag.contains(EditFlag::NULL_FRAME) {
            text.push('N');
        }
        let cstr = CString::new(text).unwrap();
        unsafe {
            SetWindowTextA(window, PCSTR(cstr.as_ptr().cast())).ok()?;
        }
        Ok(())
    }

    fn set_scroll_bar(&self, window: HWND) {
        let si = SCROLLINFO {
            cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
            fMask: SIF_DISABLENOSCROLL | SIF_PAGE | SIF_POS | SIF_RANGE,
            nMin: 0,
            nMax: (self.total_frame - 1) as i32,
            nPage: 1,
            nPos: self.frame as i32,
            nTrackPos: 0,
        };
        unsafe {
            SetScrollInfo(window, SB_HORZ, &si, true);
        }
    }
}

impl FilterPlugin for EditFilter {
    const NAME: &'static str = "簡易編集";
    const INFORMATION: &'static str = "簡易編集 version 0.07 by ＫＥＮくん";
    const WINDOW_SIZE: Size = Size {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
    };

    const FLAGS: FilterPluginFlag = FilterPluginFlag::DISP_FILTER
        .union(FilterPluginFlag::WINDOW_HORIZONTAL_SCROLL)
        .union(FilterPluginFlag::WINDOW_THICK_FRAME)
        .union(FilterPluginFlag::ALWAYS_ACTIVE)
        .union(FilterPluginFlag::WINDOW_SIZE)
        .union(FilterPluginFlag::PRIORITY_LOWEST)
        .union(FilterPluginFlag::EX_INFORMATION);

    fn handle_window(
        &mut self,
        editing: Editing,
        window: HWND,
        dll: HINSTANCE,
        message: WindowMessage,
    ) -> Result<bool> {
        match message {
            WindowMessage::System {
                original: WM_PAINT, ..
            }
            | WindowMessage::ChangeWindow => {
                self.disp(window, editing)?;
            }
            WindowMessage::Command { index } => {
                if !editing.is_editing() {
                    return Ok(false);
                }
                match index {
                    command_index::VIDEO_COPY => {
                        self.copying_frame = self.frame;
                        self.copy_mode = CopyMode::Video;
                    }
                    command_index::AUDIO_COPY => {
                        self.copying_frame = self.frame;
                        self.copy_mode = CopyMode::Audio;
                    }
                    command_index::COPY => {
                        self.copying_frame = self.frame;
                        self.copy_mode = CopyMode::All;
                    }
                    command_index::PASTE => {
                        editing.set_undo()?;
                        match self.copy_mode {
                            CopyMode::Video => {
                                editing.copy_video(self.frame, self.copying_frame)?
                            }
                            CopyMode::Audio => {
                                editing.copy_audio(self.frame, self.copying_frame)?
                            }
                            CopyMode::All => {
                                editing.copy_video_audio(self.frame, self.copying_frame)?
                            }
                        }
                        return Ok(true);
                    }
                    command_index::DELETE => {
                        editing.set_undo()?;
                        if self.frame <= self.copying_frame {
                            self.copying_frame -= 1;
                        }
                        for i in self.frame..self.total_frame - 1 {
                            editing.copy_video_audio(i, i + 1)?;
                        }
                        editing.set_total_frames(self.total_frame - 1);
                        return Ok(true);
                    }
                    command_index::INSERT => {
                        editing.set_undo()?;
                        if self.frame <= self.copying_frame {
                            self.copying_frame += 1;
                        }
                        for i in ((self.frame + 1)..=self.total_frame).rev() {
                            editing.copy_video_audio(i, i - 1)?;
                        }
                        editing.copy_video_audio(self.frame, self.copying_frame)?;
                        editing.set_total_frames(self.total_frame + 1);
                        return Ok(true);
                    }
                    command_index::FILE_INFO => {
                        use std::fmt::Write;
                        let info = editing.get_file_info()?;
                        let mut text = String::new();
                        if info.flags.contains(FileInfoFlag::VIDEO) {
                            let _ = write!(
                                text,
                                "ファイル名 : {}\nサイズ : {}x{}\nフレームレート : {:.03}fps",
                                info.name.unwrap_or_else(|| "名称未設定".into()),
                                info.size.width,
                                info.size.height,
                                info.frame_rate.as_f64(),
                            );
                        }
                        if info.flags.contains(FileInfoFlag::AUDIO) {
                            if !text.is_empty() {
                                text.push('\n');
                            }
                            let _ = write!(
                                text,
                                "サンプリングレート : {}kHz\nチャンネル数 : {}ch",
                                info.audio_rate, info.audio_channels
                            );
                        }
                        let cstr = CString::new(text).unwrap();
                        unsafe {
                            MessageBoxA(
                                window,
                                PCSTR(cstr.as_ptr().cast()),
                                PCSTR("ファイルの情報\0".as_ptr()),
                                MB_OK,
                            );
                        }
                    }
                    _ => {}
                }
            }
            WindowMessage::System {
                original: WM_HSCROLL,
                wparam,
                ..
            } => {
                let lower = wparam.0 as u32 & 0xffff;
                match lower {
                    SB_LINEDOWN => {
                        self.add_frame(1);
                    }
                    SB_LINEUP => {
                        self.sub_frame(1);
                    }
                    SB_PAGEDOWN => {
                        self.add_frame(10);
                    }
                    SB_PAGEUP => {
                        self.sub_frame(10);
                    }
                    SB_THUMBTRACK => {
                        let mut scroll = SCROLLINFO::default();
                        let mut msg = MaybeUninit::uninit();
                        unsafe {
                            GetScrollInfo(window, SB_HORZ, &mut scroll).ok()?;
                            if PeekMessageA(
                                msg.as_mut_ptr(),
                                window,
                                WM_MOUSEMOVE,
                                WM_MOUSEMOVE,
                                PM_NOREMOVE,
                            )
                            .as_bool()
                            {
                                return Ok(false);
                            }
                        }
                        self.frame = scroll.nTrackPos as usize;
                    }
                    _ => {}
                }
                self.set_scroll_bar(window);
                self.disp(window, editing)?;
            }
            WindowMessage::System {
                original: WM_KEYDOWN,
                wparam,
                ..
            } => {
                let lower = wparam.0 as u32 & 0xffff;
                match lower {
                    VK_RIGHT => {
                        self.add_frame(1);
                    }
                    VK_LEFT => {
                        self.sub_frame(1);
                    }
                    _ => {}
                }
                self.set_scroll_bar(window);
                self.disp(window, editing)?;
            }
            WindowMessage::FileOpen => {
                self.frame = 0;
                self.copying_frame = 0;
                self.copy_mode = CopyMode::All;
                self.size = editing.frame_size()?;
            }
            WindowMessage::Update => {
                if !editing.is_editing() {
                    return Ok(false);
                }
                self.total_frame = editing.total_frames();
                self.set_scroll_bar(window);
                self.disp(window, editing)?;
            }
            WindowMessage::Init => unsafe {
                let menu = LoadMenuA(dll, PCSTR("FILTER\0".as_ptr()))?;
                SetMenu(window, menu).ok()?;
                DrawMenuBar(window).ok()?;
            },
            WindowMessage::FileClose => {
                self.frame = 0;
                self.total_frame = 0;
                self.set_scroll_bar(window);
                self.disp(window, editing)?;
            }
            _ => {}
        }
        Ok(false)
    }

    fn load_project(&mut self, _editing: Editing, mut load: &[u8]) -> Result<()> {
        use std::io::Read;
        load.read(&mut self.example_data)
            .map_err(AviUtlError::Load)?;
        Ok(())
    }

    fn save_project(&self, _editing: Editing, mut save: &mut [u8]) -> Result<usize> {
        if save.len() < 3 * std::mem::size_of::<u32>() {
            return Err(AviUtlError::BufferLimitExceed);
        }
        use std::io::Write;
        save.write(&self.example_data).map_err(AviUtlError::Save)
    }
}

export_filter_plugin!(EditFilter);
