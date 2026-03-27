pub mod errors;

pub struct PanicManager {}

impl PanicManager {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn bug_check(&mut self, code: errors::ErrorTypeEnum) -> ! {
        loop {}
    }
}
