use alloc::vec::Vec;
use bootloader_api::info::FrameBufferInfo;

pub enum DisplayError {}

pub struct DisplayManager {
    main_display: Option<Display>,
    other_displays: Option<Vec<Display>>,
}

impl DisplayManager {
    pub const fn new() -> Self {
        Self {
            main_display: None,
            other_displays: None,
        }
    }

    pub fn add_display(&mut self, display: Display) -> Result<(), DisplayError> {
        if self.main_display.is_none() {
            self.main_display = Some(display);
        } else {
            self.other_displays
                .get_or_insert_with(Vec::new)
                .push(display);
        }

        Ok(())
    }

    pub fn get_display(&mut self, index: usize) -> Option<&mut Display> {
        if index == 0 {
            self.main_display.as_mut()
        } else {
            self.other_displays.as_mut()?.get_mut(index - 1)
        }
    }

    pub fn all_displays(&mut self) -> Option<Vec<&mut Display>> {
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

pub struct Display {
    pub buffer: &'static mut [u8],
    pub info: FrameBufferInfo,
}

impl Display {
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        Self { buffer, info }
    }
}
