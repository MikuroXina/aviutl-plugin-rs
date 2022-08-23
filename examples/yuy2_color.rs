//! Example from サンプルYUY2色変換プラグイン(色変換プラグイン) for AviUtl ver0.99h or later by ＫＥＮくん.

use aviutl_plugin::{color::prelude::*, PixelFormat};

#[derive(Debug, Default)]
struct Yuy2Plugin;

impl ColorPlugin for Yuy2Plugin {
    const NAME: &'static str = "サンプルYUY2変換";
    const INFORMATION: &'static str = "サンプルYUY2変換 version 0.01 By ＫＥＮくん";
    fn pixel_to_yc(&mut self, proc_info: &ProcInfo, from: &[u8], to: &mut [PixelYc]) -> Result<()> {
        if proc_info.format != PixelFormat::from_four_code([b'Y', b'U', b'Y', b'2']) {
            return Err(AviUtlError::Unsupported("supported only YUY2".into()));
        }

        // Assume `from` as ITU-R BT.601.
        for y in 0..proc_info.size.height as usize {
            let pixel_p = &from[y * ((proc_info.size.width as usize + 1) / 2 * 4)..];
            let yc_p = &mut to[y * proc_info.line_bytes..];
            for x in (0..)
                .map(|x| x * 2)
                .take_while(|&x| x < proc_info.size.width as usize)
            {
                yc_p[x] = PixelYc {
                    y: ((pixel_p[2 * x] as i32 - 16) * 4096 / 219) as i16,
                    cb: ((pixel_p[2 * x + 1] as i32 - 128) * 4096 / 224) as i16,
                    cr: ((pixel_p[2 * x + 3] as i32 - 128) * 4096 / 224) as i16,
                };
                yc_p[x + 1] = PixelYc {
                    y: ((pixel_p[2 * x + 2] as i32 - 16) * 4096 / 219) as i16,
                    cb: ((pixel_p[2 * x + 1] as i32 - 128) * 4096 / 224) as i16,
                    cr: ((pixel_p[2 * x + 3] as i32 - 128) * 4096 / 224) as i16,
                }
            }
        }
        Ok(())
    }
    fn yc_to_pixel(&mut self, proc_info: &ProcInfo, from: &[PixelYc], to: &mut [u8]) -> Result<()> {
        fn pack_byte(val: i32) -> u8 {
            val.clamp(u8::MIN as i32, u8::MAX as i32) as u8
        }
        if proc_info.format != PixelFormat::from_four_code([b'Y', b'U', b'Y', b'2']) {
            return Err(AviUtlError::Unsupported("supported only YUY2".into()));
        }

        // Assume `to` as ITU-R BT.601.
        for y in 0..proc_info.size.height as usize {
            let yc_p = &from[y * proc_info.line_bytes..];
            let pixel_p = &mut to[y * ((proc_info.size.width as usize + 1) / 2 * 4)..];
            for x in (0..)
                .map(|x| x * 2)
                .take_while(|&x| x < proc_info.size.width as usize)
            {
                pixel_p[2 * x] = pack_byte(yc_p[x].y as i32 * 219 / 4096 + 16);
                pixel_p[2 * x + 1] = pack_byte(yc_p[x].cb as i32 * 224 / 4096 + 128);
                pixel_p[2 * x + 3] = pack_byte(yc_p[x].cr as i32 * 224 / 4096 + 128);
                pixel_p[2 * x + 2] = pack_byte(yc_p[x + 1].y as i32 * 219 / 4096 + 16);
            }
        }
        Ok(())
    }
}

export_color_plugin!(Yuy2Plugin);
