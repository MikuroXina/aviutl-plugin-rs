//! Example from サンプルBMP出力プラグイン (出力プラグイン) for AviUtl ver0.98 or later by ＫＥＮくん.

use aviutl_plugin::{output::prelude::*, PixelFormat};
use std::{
    mem::size_of,
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicU8, Ordering},
};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{BOOL, HINSTANCE, HWND, LPARAM, WPARAM},
        Graphics::Gdi::{BITMAPFILEHEADER, BITMAPINFOHEADER, BI_RGB},
        UI::WindowsAndMessaging::{
            DialogBoxParamA, EndDialog, GetDlgItemInt, SetDlgItemInt, IDCANCEL, IDOK,
            MESSAGEBOX_RESULT, WM_COMMAND, WM_INITDIALOG,
        },
    },
};

const IDC_EDIT0: i32 = 100;

#[derive(Debug)]
struct BmpOutput {
    num_width: u8,
}

impl Default for BmpOutput {
    fn default() -> Self {
        Self { num_width: 4 }
    }
}

impl OutputPlugin for BmpOutput {
    const NAME: &'static str = "連番BMP出力";
    const INFORMATION: &'static str = "連番BMP出力 version 0.06 By ＫＥＮくん";
    fn file_filters() -> FileFilters {
        let mut filters = FileFilters::new();
        filters.add_filter("BMP File (*.bmp)", "*.bmp");
        filters.add_filter("All File (*.*)", "*.*");
        filters
    }
    fn output(&mut self, info: Info) -> Result<()> {
        let file_header = BITMAPFILEHEADER {
            bfType: u16::from_le_bytes([b'B', b'M']),
            bfOffBits: (size_of::<BITMAPFILEHEADER>() + size_of::<BITMAPINFOHEADER>()) as u32,
            bfSize: (size_of::<BITMAPFILEHEADER>()
                + size_of::<BITMAPINFOHEADER>()
                + info.video_bytes_per_frame) as u32,
            ..BITMAPFILEHEADER::default()
        };
        let info_header = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: info.size.width as i32,
            biHeight: info.size.height as i32,
            biPlanes: 1,
            biBitCount: 24,
            biCompression: BI_RGB as u32,
            ..BITMAPINFOHEADER::default()
        };

        let save_path = PathBuf::from_str(&info.save_file).unwrap();
        let save_file_name = save_path.file_name().expect("file name not specified");
        for i in 0..info.video_frames {
            if info.is_aborted() {
                break;
            }
            info.show_remaining_time(i, info.video_frames)?;
            let pixels = info.get_video_ex(i, PixelFormat::default());
            let index_postfix = format!("{:0width$}", i, width = self.num_width as usize);
            let mut indexed_file_name = save_file_name.to_owned();
            indexed_file_name.push(index_postfix);
            let indexed_path = save_path.with_file_name(indexed_file_name);
            let mut bmp_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(indexed_path)
                .map_err(AviUtlError::File)?;
            use std::io::Write;
            bmp_file
                .write_all(unsafe { struct_to_bytes_helper(&file_header) })
                .map_err(AviUtlError::File)?;
            bmp_file
                .write_all(unsafe { struct_to_bytes_helper(&info_header) })
                .map_err(AviUtlError::File)?;
            bmp_file.write_all(pixels).map_err(AviUtlError::File)?;
            info.update_preview()?;
        }
        Ok(())
    }
    fn config_dialog(&mut self, window: HWND, dll: HINSTANCE) -> Result<()> {
        static NUM_WIDTH_INPUT: AtomicU8 = AtomicU8::new(0);

        unsafe extern "system" fn handler(
            window: HWND,
            message: u32,
            wparam: WPARAM,
            _lparam: LPARAM,
        ) -> isize {
            match message {
                WM_INITDIALOG => {
                    SetDlgItemInt(
                        window,
                        IDC_EDIT0,
                        NUM_WIDTH_INPUT.load(Ordering::Relaxed) as u32,
                        BOOL::default(),
                    );
                    1
                }
                WM_COMMAND => {
                    let lower = wparam.0 as u16;
                    match MESSAGEBOX_RESULT(lower as i32) {
                        IDCANCEL => {
                            EndDialog(window, lower as isize);
                        }
                        IDOK => {
                            let value =
                                GetDlgItemInt(window, IDC_EDIT0, std::ptr::null_mut(), false);
                            if 0 < value {
                                NUM_WIDTH_INPUT
                                    .store(value.try_into().unwrap_or(u8::MAX), Ordering::Relaxed);
                            }
                            EndDialog(window, lower as isize);
                        }
                        _ => {}
                    }
                    0
                }
                _ => 0,
            }
        }

        NUM_WIDTH_INPUT.store(self.num_width, Ordering::Relaxed);
        unsafe {
            DialogBoxParamA(
                dll,
                PCSTR::from_raw(b"CONFIG\0" as *const _),
                window,
                Some(handler),
                LPARAM::default(),
            );
        }
        self.num_width = NUM_WIDTH_INPUT.load(Ordering::Relaxed);
        Ok(())
    }
    fn config_get(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Err(AviUtlError::BufferLimitExceed);
        }
        self.num_width = buf[0];
        Ok(1)
    }
    fn config_set(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Err(AviUtlError::BufferLimitExceed);
        }
        buf[0] = self.num_width;
        Ok(1)
    }
}

export_output_plugin!(BmpOutput);

unsafe fn struct_to_bytes_helper<T>(str: &T) -> &[u8] {
    std::slice::from_raw_parts(str as *const T as *const u8, size_of::<T>())
}
