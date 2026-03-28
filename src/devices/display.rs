use alloc::vec::Vec;
use bootloader_api::info::FrameBufferInfo;
use embedded_graphics::{
    Pixel,
    pixelcolor::Rgb888,
    prelude::{DrawTarget, OriginDimensions, RgbColor, Size},
};

pub enum DisplayError {}

pub struct DisplayManager<'a> {
    main_display: Option<Display<'a>>,
    other_displays: Option<Vec<Display<'a>>>,
}

impl<'a> DisplayManager<'a> {
    pub const fn new() -> Self {
        Self {
            main_display: None,
            other_displays: None,
        }
    }

    pub fn add_display(&mut self, display: Display<'a>) -> Result<(), DisplayError> {
        if self.main_display.is_none() {
            self.main_display = Some(display);
        } else {
            self.other_displays
                .get_or_insert_with(Vec::new)
                .push(display);
        }

        Ok(())
    }

    pub fn get_display(&mut self, index: usize) -> Option<&mut Display<'a>> {
        if index == 0 {
            self.main_display.as_mut()
        } else {
            self.other_displays.as_mut()?.get_mut(index - 1)
        }
    }

    pub fn all_displays(&mut self) -> Option<Vec<&mut Display<'a>>> {
        let mut displays = Vec::new();
        if let Some(main_display) = &mut self.main_display {
            displays.push(main_display);
            displays.extend(self.other_displays.iter_mut().flat_map(|v| v.iter_mut()));
            Some(displays)
        } else {
            None
        }
    }
}

pub struct Display<'a> {
    framebuffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> Display<'a> {
    pub const fn new(framebuffer: &'a mut [u8], info: FrameBufferInfo) -> Self {
        Self { framebuffer, info }
    }
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.info.width;
        let height = self.info.height;

        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x, y)) = coord.try_into() {
                let (x, y): (u32, u32) = (x, y);
                let (x, y): (usize, usize) = (x as usize, y as usize);

                if x >= width || y >= height {
                    continue;
                }

                let index = ((y * width + x) * self.info.bytes_per_pixel) as usize;
                let buffer = [color.b(), color.g(), color.r()];
                self.framebuffer[index..index + 3].copy_from_slice(&buffer);
            }
        }

        Ok(())
    }
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        return Size::new(self.info.width as u32, self.info.height as u32);
    }
}
