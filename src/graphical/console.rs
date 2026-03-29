use core::{fmt, fmt::Write};

use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};

use crate::graphical::framebuffer::FrameBuffer;

struct Cursor {
    maximum_width: u32,
    maximum_height: u32,
    width: u32,
    height: u32,
}

impl Cursor {
    pub fn new(maximum_width: u32, maximum_height: u32) -> Self {
        Self {
            maximum_width,
            maximum_height,
            width: 0,
            height: 0,
        }
    }
}

pub struct Console<'a> {
    font: &'a MonoFont<'a>,
    framebuffer: &'a mut FrameBuffer<'a>,

    bounds: Rectangle,
    cursor: Cursor,

    character_size: Size,

    background: Rgb888,
}

impl<'a> Console<'a> {
    pub fn new(
        font: &'a MonoFont<'a>,
        framebuffer: &'a mut FrameBuffer<'a>,
        position: Point,
        size: Size,
        background: Rgb888,
    ) -> Self {
        let w = font.character_size.width;
        let h = font.character_size.height + font.character_spacing;
        let mcw = size.width / w;
        let mch = size.height / h;

        let bounds = Rectangle::new(position, size);
        framebuffer.fill_solid(&bounds, Rgb888::new(0, 0, 0));

        Self {
            font,
            framebuffer,
            bounds,
            cursor: Cursor::new(mcw, mch),
            character_size: Size::new(w, h),
            background,
        }
    }

    pub fn get_pixel_position(&self) -> Point {
        Point::new(
            self.bounds.top_left.x + (self.cursor.width * self.character_size.width) as i32,
            self.bounds.top_left.y + (self.cursor.height * self.character_size.height) as i32,
        )
    }

    pub fn draw_text(&mut self, text: &str, foreground: Rgb888) -> Result<(), ()> {
        for ch in text.chars() {
            if ch == '\n' {
                self.cursor.width = 0;
                self.cursor.height += 1;
            } else {
                let pos = self.get_pixel_position();

                let mut char_buf = [0u8; 4];
                let char_str = ch.encode_utf8(&mut char_buf);

                let style = MonoTextStyleBuilder::new()
                    .font(self.font)
                    .text_color(foreground)
                    .background_color(self.background)
                    .build();

                let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

                Text::with_text_style(char_str, pos, style, text_style)
                    .draw(self.framebuffer)
                    .map_err(|_| ())?;

                self.cursor.width += 1;

                if self.cursor.width >= self.cursor.maximum_width {
                    self.cursor.width = 0;
                    self.cursor.height += 1;
                }
            }

            if self.cursor.height >= self.cursor.maximum_height {
                self.scroll_up();
            }
        }

        Ok(())
    }

    fn scroll_up(&mut self) {
        let line_h = self.character_size.height;
        let total_lines = self.cursor.maximum_height;
        let total_width = self.bounds.size.width;

        for line in 1..total_lines {
            for x in 0..total_width {
                let src = Point::new(
                    self.bounds.top_left.x + x as i32,
                    self.bounds.top_left.y + (line * line_h) as i32,
                );
                let dst = Point::new(
                    self.bounds.top_left.x + x as i32,
                    self.bounds.top_left.y + ((line - 1) * line_h) as i32,
                );
                let pixel = self.framebuffer.get_pixel(src);
                self.framebuffer.set_pixel(dst, pixel.unwrap());
            }
        }

        let clear_style = PrimitiveStyleBuilder::new()
            .fill_color(self.background)
            .build();

        Rectangle::new(
            Point::new(
                self.bounds.top_left.x,
                self.bounds.top_left.y + ((total_lines - 1) * line_h) as i32,
            ),
            Size::new(total_width, line_h),
        )
        .into_styled(clear_style)
        .draw(self.framebuffer)
        .ok();

        self.cursor.height = total_lines - 1;
    }

    pub fn update_buffer(&mut self, buffer: &'a mut FrameBuffer<'a>) {
        self.framebuffer = buffer;
        self.update_console(self.font, self.bounds.top_left, self.bounds.size);
    }

    pub fn update_font(&mut self, font: &'a MonoFont<'a>) {
        self.update_console(font, self.bounds.top_left, self.bounds.size);
    }

    pub fn update_position(&mut self, position: Point) {
        self.update_console(self.font, position, self.bounds.size);
    }

    pub fn update_size(&mut self, size: Size) {
        self.update_console(self.font, self.bounds.top_left, size);
    }

    pub fn update_console(&mut self, font: &'a MonoFont<'a>, position: Point, size: Size) {
        let w = font.character_size.width;
        let h = font.character_size.height + font.character_spacing;
        let mcw = size.width / w;
        let mch = size.height / h;

        self.font = font;
        self.bounds = Rectangle::new(position, size);
        self.cursor = Cursor::new(mcw, mch);
        self.character_size = Size::new(w, h);
    }
}

impl Write for Console<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self.draw_text(s, Rgb888::WHITE) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
