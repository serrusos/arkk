use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    Pixel,
    geometry::OriginDimensions,
    pixelcolor::{Rgb888, RgbColor},
    prelude::{DrawTarget, Point, Size},
};

pub struct FrameBuffer<'a> {
    pub buffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> FrameBuffer<'a> {
    pub fn new(buffer: &'a mut [u8], info: FrameBufferInfo) -> Self {
        Self { buffer, info }
    }

    pub fn is_out_of_bounds(&self, position: Point) -> bool {
        // Also guard against negative coordinates
        if position.x < 0 || position.y < 0 {
            return true;
        }
        let (x, y) = (position.x as usize, position.y as usize);
        x >= self.info.width || y >= self.info.height
    }

    fn pixel_to_bytes(&self, pixel: Rgb888) -> [u8; 3] {
        // FIX: respect the actual pixel format reported by the bootloader
        match self.info.pixel_format {
            PixelFormat::Rgb => [pixel.r(), pixel.g(), pixel.b()],
            PixelFormat::Bgr => [pixel.b(), pixel.g(), pixel.r()],
            // U8 and Unknown: best-effort, write grayscale luminance
            _ => {
                let luma = ((pixel.r() as u16 + pixel.g() as u16 + pixel.b() as u16) / 3) as u8;
                [luma, luma, luma]
            }
        }
    }

    pub fn get_pixel(&self, source: Point) -> Option<Rgb888> {
        // FIX: was missing bounds check entirely
        if self.is_out_of_bounds(source) {
            return None;
        }
        let (x, y) = (source.x as usize, source.y as usize);
        let index = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        Some(match self.info.pixel_format {
            PixelFormat::Rgb => Rgb888::new(
                self.buffer[index],
                self.buffer[index + 1],
                self.buffer[index + 2],
            ),
            // FIX: BGR stored as B, G, R — reconstruct as R, G, B
            PixelFormat::Bgr => Rgb888::new(
                self.buffer[index + 2],
                self.buffer[index + 1],
                self.buffer[index],
            ),
            _ => Rgb888::new(self.buffer[index], self.buffer[index], self.buffer[index]),
        })
    }

    pub fn set_pixel(&mut self, destination: Point, pixel: Rgb888) {
        if self.is_out_of_bounds(destination) {
            return;
        }
        let (x, y) = (destination.x as usize, destination.y as usize);
        // FIX: use stride instead of width — framebuffers may have padding between rows
        let index = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        let pixel_bytes = self.pixel_to_bytes(pixel);
        // FIX: was `index + 2` (2 bytes), must be `index + 3` (3 bytes) to match pixel_bytes len
        self.buffer[index..index + 3].copy_from_slice(&pixel_bytes);
    }
}

impl<'a> DrawTarget for FrameBuffer<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if self.is_out_of_bounds(coord) {
                continue;
            }
            self.set_pixel(coord, color);
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for FrameBuffer<'a> {
    fn size(&self) -> Size {
        Size::new(self.info.width as u32, self.info.height as u32)
    }
}
