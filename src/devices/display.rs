use alloc::vec::Vec;
use embedded_graphics::{
    Pixel,
    pixelcolor::Rgb888,
    prelude::{DrawTarget, OriginDimensions, RgbColor, Size},
};

enum DisplayError {}

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
}

pub struct Display<'a> {
    framebuffer: &'a mut [u8],
    buffer_dimensions: Size,
}

impl<'a> Display<'a> {
    pub const fn new(framebuffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            framebuffer,
            buffer_dimensions: Size::new(width, height),
        }
    }
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=63, y @ 0..=63)) = coord.try_into() {
                // Calculate the index in the framebuffer.
                let index: u32 = x + y * 64;

                self.framebuffer[index as usize] = color.r();
                self.framebuffer[index as usize + 1] = color.g();
                self.framebuffer[index as usize + 2] = color.b();
            }
        }

        Ok(())
    }
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        return self.buffer_dimensions;
    }
}
