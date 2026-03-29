#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod devices;
pub mod graphical;
pub mod panic;

struct Allocator {}

impl Allocator {
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!();
    }

    unsafe fn dealloc(&self, _address: *mut u8, _layout: core::alloc::Layout) {
        todo!();
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

use core::{alloc::GlobalAlloc, fmt::Write};

use crate::{
    devices::display::{Display, DisplayManager},
    graphical::{console::Console, framebuffer::FrameBuffer},
    panic::PanicManager,
};
use bootloader_api::{BootInfo, entry_point};
use embedded_graphics::{
    mono_font::jis_x0201::FONT_10X20,
    pixelcolor::Rgb888,
    prelude::{OriginDimensions, Point},
};

use spin::Mutex;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};
use x86_64::instructions::interrupts::int3;

static DISPLAY_MANAGER: Mutex<DisplayManager> = Mutex::new(DisplayManager::new());
static PANIC_MANAGER: Mutex<PanicManager> = Mutex::new(PanicManager::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

pub fn serial() -> Uart16550Tty<PioBackend> {
    unsafe { Uart16550Tty::new_port(0x3F8, Config::default()).expect("Should initialize device") }
}

fn main(boot_info: &'static mut BootInfo) -> ! {
    {
        let manager = PANIC_MANAGER.lock();
        manager.inject_table();
    }

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let display = Display::new(framebuffer.buffer_mut(), info.clone());
        DISPLAY_MANAGER.lock().add_display(display);
    }

    let mut display_manager = DISPLAY_MANAGER.lock();
    let display = display_manager.get_display(0).unwrap();

    let mut framebuffer = FrameBuffer::new(display.buffer, display.info);
    let size = framebuffer.size();

    let mut console = Console::new(
        &FONT_10X20,
        &mut framebuffer,
        Point::new(0, 0),
        size,
        Rgb888::new(0, 0, 0),
    );
    writeln!(console, "It's now safe to turn off your computer.").unwrap();
    writeln!(console, "").unwrap();
    writeln!(
        console,
        "If you want to restart your computer, press CTRL+ALT+DEL."
    )
    .unwrap();

    int3();
    unsafe { *(0xdeadbeef as *mut u8) = 0 }

    loop {}
}

entry_point!(main);
