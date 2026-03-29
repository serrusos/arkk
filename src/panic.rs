pub mod errors;
mod idt;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    mono_font::{MonoTextStyle, iso_8859_5::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::{Dimensions, Drawable, Point, RgbColor},
    primitives::Rectangle,
    text::Text,
};

use crate::{
    PANIC_MANAGER, graphical::framebuffer::FrameBuffer, panic::errors::ErrorTypeEnum, serial,
};

use core::fmt::Write;

use super::DISPLAY_MANAGER;

pub struct PanicManager {}

impl PanicManager {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn inject_table(&self) {
        idt::load();
    }

    pub fn bug_check(
        &mut self,
        code: errors::ErrorTypeEnum,
        parameter_1: Option<*const u32>,
        parameter_2: Option<*const u32>,
        parameter_3: Option<*const u32>,
        parameter_4: Option<*const u32>,
    ) -> ! {
        let mut port = serial();
        writeln!(port, "Locking display manager").unwrap();
        unsafe { DISPLAY_MANAGER.force_unlock() };
        let mut manager = DISPLAY_MANAGER.lock();

        writeln!(port, "Reading main display").unwrap();

        if let Some(display) = manager.get_display(0) {
            writeln!(port, "Creating new framebuffer").unwrap();

            let mut framebuffer = FrameBuffer::new(display.buffer, display.info);
            let size = framebuffer.size();
            let rect = Rectangle::new(Point::new(0, 0), size);

            writeln!(port, "Refreshing").unwrap();
            framebuffer.fill_solid(&rect, Rgb888::new(0, 0, 0));

            writeln!(port, "Formatting stop code").unwrap();
            let mut buf = [0u8; 128];
            let s = format_no_std::show(
                &mut buf,
                format_args!(
                    "Stop code: {} 0x{:08x} (0x{:08x}, 0x{:08x}, 0x{:08x}, 0x{:08x})",
                    code.as_str(),
                    code.as_code(),
                    parameter_1.unwrap_or(0 as *const u32) as u32,
                    parameter_2.unwrap_or(0 as *const u32) as u32,
                    parameter_3.unwrap_or(0 as *const u32) as u32,
                    parameter_4.unwrap_or(0 as *const u32) as u32,
                ),
            );

            let style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);

            let mut btext = Text::new(s.unwrap(), Point::new(0, 0), style);
            let mut mtext = Text::new(
                "Your device ran into a problem and needs to restart.",
                Point::new(0, 0),
                style,
            );

            let bbb = btext.bounding_box();
            let mbb = mtext.bounding_box();

            btext.position = Point::new(
                ((size.width - bbb.size.width) / 2) as i32,
                (size.height - bbb.size.height - 10) as i32,
            );
            mtext.position = Point::new(
                ((size.width - mbb.size.width) / 2) as i32,
                ((size.height - mbb.size.height) / 2) as i32,
            );

            btext.draw(&mut framebuffer).unwrap();
            mtext.draw(&mut framebuffer).unwrap();
        }

        writeln!(port, "Failed").unwrap();

        loop {}
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let mut panic_manager = PANIC_MANAGER.lock();
    panic_manager.bug_check(
        ErrorTypeEnum::KernelModeExceptionNotHandled,
        None,
        None,
        None,
        None,
    );
}
