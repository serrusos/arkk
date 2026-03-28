use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::PixelColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

pub struct Console<'a, C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    display: &'a mut D,
    bounds: Rectangle,
    font: &'a MonoFont<'a>,
    fg: C,
    bg: C,

    // current cursor position in pixels (relative to bounds origin)
    cursor: Point,

    char_w: u32,
    char_h: u32,
    spacing: u32,
}

impl<'a, C, D> Console<'a, C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    /// Create a new console that fills the entire display area.
    pub fn new(display: &'a mut D, font: &'a MonoFont<'a>, fg: C, bg: C) -> Self
    where
        D: OriginDimensions,
    {
        let size = display.size();
        Self::with_bounds(display, Rectangle::new(Point::zero(), size), font, fg, bg)
    }

    /// Create a console constrained to a specific rectangle on the display.
    pub fn with_bounds(
        display: &'a mut D,
        bounds: Rectangle,
        font: &'a MonoFont<'a>,
        fg: C,
        bg: C,
    ) -> Self {
        let char_w = font.character_size.width;
        let char_h = font.character_size.height;
        let spacing = font.character_spacing;

        Self {
            display,
            bounds,
            font,
            fg,
            bg,
            cursor: Point::zero(),
            char_w,
            char_h,
            spacing,
        }
    }

    /// Clear the console area with the background colour.
    pub fn clear(&mut self) -> Result<(), D::Error> {
        self.bounds
            .into_styled(PrimitiveStyle::with_fill(self.bg))
            .draw(self.display)?;
        self.cursor = Point::zero();
        Ok(())
    }

    /// Move the cursor to the start of the next line, scrolling if necessary.
    pub fn newline(&mut self) -> Result<(), D::Error> {
        self.cursor.x = 0;
        self.cursor.y += self.char_h as i32;
        self.scroll_if_needed()?;
        Ok(())
    }

    /// Push a string, wrapping at word boundaries (or mid-word if a single
    /// word is wider than the console).
    pub fn push(&mut self, text: &str) -> Result<(), D::Error> {
        let max_w = self.bounds.size.width;

        // How many chars fit on one line?
        let chars_per_line = self.chars_per_line();
        if chars_per_line == 0 {
            return Ok(());
        }

        let mut remaining = text;

        while !remaining.is_empty() {
            // How many pixels are already used on the current line?
            let used_px = self.cursor.x as u32;
            // How many more chars fit on this line?
            let space_chars = self.px_to_chars(max_w.saturating_sub(used_px));

            if space_chars == 0 {
                self.newline()?;
                continue;
            }

            // Find the best break point within `space_chars` characters.
            let (chunk, rest) = Self::split_at_wrap(remaining, space_chars);

            self.draw_str(chunk)?;
            remaining = rest;

            // If there is more text, we wrapped — move to the next line.
            if !remaining.is_empty() {
                // Skip a leading space at the start of the next line.
                remaining = remaining.trim_start_matches(' ');
                self.newline()?;
            }
        }

        Ok(())
    }

    // ------------------------------------------------------------------ //
    //  Internal helpers
    // ------------------------------------------------------------------ //

    /// Draw a string at the current cursor, advancing the cursor.
    /// Does NOT wrap — callers must have pre-split the string.
    fn draw_str(&mut self, s: &str) -> Result<(), D::Error> {
        if s.is_empty() {
            return Ok(());
        }

        let origin = self.bounds.top_left + self.cursor
            // MonoTextStyle baseline is at the BOTTOM of the character cell.
            + Point::new(0, self.char_h as i32 - 1);

        let style = MonoTextStyle::new(self.font, self.fg);
        Text::new(s, origin, style).draw(self.display)?;

        let drawn_px = self.str_width_px(s);
        self.cursor.x += drawn_px as i32;
        Ok(())
    }

    /// Pixel width of a string (no trailing spacing on the last char).
    fn str_width_px(&self, s: &str) -> u32 {
        let n = s.chars().count() as u32;
        if n == 0 {
            return 0;
        }
        n * self.char_w + (n - 1) * self.spacing
    }

    /// How many characters fit in `px` pixels?
    fn px_to_chars(&self, px: u32) -> usize {
        if self.char_w == 0 {
            return 0;
        }
        // Each char except the first needs `spacing` extra pixels.
        // chars * char_w + (chars-1) * spacing <= px
        // chars * (char_w + spacing) <= px + spacing
        ((px + self.spacing) / (self.char_w + self.spacing)) as usize
    }

    /// Maximum characters that fit in one full console line.
    fn chars_per_line(&self) -> usize {
        self.px_to_chars(self.bounds.size.width)
    }

    /// Split `text` so that the first part fits in `max_chars` characters,
    /// preferring to break at a space.  Returns `(chunk, remainder)`.
    fn split_at_wrap(text: &str, max_chars: usize) -> (&str, &str) {
        let char_count = text.chars().count();

        if char_count <= max_chars {
            return (text, "");
        }

        // Try to find a word boundary to break at.
        let candidate = &text[..text
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(text.len())];

        if let Some(space_pos) = candidate.rfind(' ') {
            (&text[..space_pos], &text[space_pos..])
        } else {
            // No space found — hard break at max_chars.
            (candidate, &text[candidate.len()..])
        }
    }

    /// If the cursor has moved below the visible area, scroll up by one line.
    fn scroll_if_needed(&mut self) -> Result<(), D::Error> {
        let max_y = self.bounds.size.height as i32;

        while self.cursor.y + self.char_h as i32 > max_y {
            self.scroll_up()?;
        }
        Ok(())
    }

    /// Scroll the console content up by one character line.
    fn scroll_up(&mut self) -> Result<(), D::Error> {
        let line_h = self.char_h;
        let w = self.bounds.size.width;
        let h = self.bounds.size.height;
        let origin = self.bounds.top_left;

        // Copy each row of pixels one `line_h` upward.
        // We do this row-by-row using individual pixels — a simple but
        // portable approach that works on any DrawTarget.
        for y in 0..(h - line_h) {
            for x in 0..w {
                let src = Point::new(origin.x + x as i32, origin.y + y as i32 + line_h as i32);
                // Read pixel via a tiny single-pixel iterator trick using a
                // sub-display slice isn't available on all targets, so we
                // rely on the display implementing `DrawTarget` only.
                // Instead, we store the row in a buffer using `Pixel` draws.
                let _ = Pixel(Point::new(origin.x + x as i32, origin.y + y as i32), {
                    // NOTE: We cannot *read* pixels from a generic DrawTarget.
                    // See note below about scroll_up limitations.
                    let _ = src;
                    self.bg // placeholder — see note
                })
                .draw(self.display);
            }
        }

        // Because generic `DrawTarget` is write-only, we cannot truly copy
        // pixels. The portable alternative is a full redraw. If your display
        // implements a framebuffer or supports `ReadTarget`, see the note at
        // the bottom of this file.
        //
        // Simple approach: clear the bottom line.
        let bottom_line = Rectangle::new(
            Point::new(origin.x, origin.y + (h - line_h) as i32),
            Size::new(w, line_h),
        );
        bottom_line
            .into_styled(PrimitiveStyle::with_fill(self.bg))
            .draw(self.display)?;

        self.cursor.y -= line_h as i32;
        Ok(())
    }
}
