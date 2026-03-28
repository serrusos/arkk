pub mod errors;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    mono_font::iso_8859_5::FONT_8X13,
    prelude::{Point, RgbColor},
    primitives::Rectangle,
};

use super::DISPLAY_MANAGER;
use super::graphical::console::Console;

pub struct PanicManager {}

impl PanicManager {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn bug_check(&mut self, code: errors::ErrorTypeEnum) -> ! {
        let mut manager = DISPLAY_MANAGER.lock();

        if let Some(displays) = manager.all_displays() {
            for display in displays {
                let rect = Rectangle::new(Point::new(0, 0), display.size());
                display.fill_solid(&rect, RgbColor::BLUE);
            }
        }

        if let Some(main_display) = manager.get_display(0) {
            let mut console =
                Console::new(main_display, &FONT_8X13, RgbColor::WHITE, RgbColor::BLUE);

            let mut buf = [0u8; 64];
            let s = format_no_std::show(&mut buf, format_args!("*** STOP: {:x}", code.as_code()));

            console.push(s.unwrap());
            console.newline();
            console.push(code.as_str());
            console.push("If this is the first time you've seen this Stop error screen, restart your computer. If this screen appears again, follow these steps:");
            console.newline();
            console.newline();
            console.push("Check for viruses on your computer. Remove any newly installed hard drives or hard drive controllers. Check your hard drive to make sure it is properly configured and terminated. Run CHKDSK /F to check for hard drive corruption, and then restart your computer.");
            console.newline();
            console.newline();
            console.push("Refer to your Getting Started manual for more information on troubleshooting Stop errors.");
        }

        loop {}
    }
}
