#![no_std]
#![no_main]

extern crate alloc;

mod devices;
mod panic;

use crate::{
    devices::display::{Display, DisplayManager},
    panic::PanicManager,
};
use bootloader_api::{BootInfo, entry_point};
use embedded_graphics::{
    prelude::{DrawTarget, OriginDimensions, Point, RgbColor},
    primitives::Rectangle,
};
use panic::errors::ErrorTypeEnum;
use spin::Mutex;

static DISPLAY_MANAGER: Mutex<DisplayManager> = Mutex::new(DisplayManager::new());
static PANIC_MANAGER: Mutex<PanicManager> = Mutex::new(PanicManager::new());

fn main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let display = Display::new(
            framebuffer.buffer_mut(),
            info.width as u32,
            info.height as u32,
        );

        DISPLAY_MANAGER.lock().add_display(display);
    }

    let mut display_manager = DISPLAY_MANAGER.lock();
    let display = display_manager.get_display(0).unwrap();

    let rect = Rectangle::new(Point::new(0, 0), display.size());
    display.fill_solid(&rect, RgbColor::BLUE);

    let mut panic_manager = PANIC_MANAGER.lock();
    panic_manager.bug_check(ErrorTypeEnum::ManuallyInitiatedCrash);
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

entry_point!(main);
