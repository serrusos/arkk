use lazy_static::lazy_static;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::{PANIC_MANAGER, panic::errors::ErrorTypeEnum};

extern "x86-interrupt" fn divide_error(stack_frame: InterruptStackFrame) {
    match stack_frame.code_segment.rpl() {
        x86_64::PrivilegeLevel::Ring0 => {}
        _ => unsafe { stack_frame.iretq() },
    }

    let mut manager = PANIC_MANAGER.lock();
    let value = 0x0;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
    match stack_frame.code_segment.rpl() {
        x86_64::PrivilegeLevel::Ring0 => {}
        _ => unsafe { stack_frame.iretq() },
    }

    let mut manager = PANIC_MANAGER.lock();
    let value = 0x4;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
    match stack_frame.code_segment.rpl() {
        x86_64::PrivilegeLevel::Ring0 => {}
        _ => unsafe { stack_frame.iretq() },
    }

    let mut manager = PANIC_MANAGER.lock();
    let value = 0x5;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    match stack_frame.code_segment.rpl() {
        x86_64::PrivilegeLevel::Ring0 => {}
        _ => unsafe { stack_frame.iretq() },
    }

    let mut manager = PANIC_MANAGER.lock();
    let value = 0x6;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn double_fault(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    let mut manager = PANIC_MANAGER.lock();
    let value = 0x8;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn invalid_tss(_stack_frame: InterruptStackFrame, _error_code: u64) {
    let mut manager = PANIC_MANAGER.lock();
    let value = 0xa;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    match stack_frame.code_segment.rpl() {
        x86_64::PrivilegeLevel::Ring0 => {}
        _ => unsafe { stack_frame.iretq() },
    }

    let mut manager = PANIC_MANAGER.lock();
    let value = 0x3;
    manager.bug_check(
        ErrorTypeEnum::UnexpectedKernelModeTrap,
        Some(&value),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn page_fault(
    _stack_frame: InterruptStackFrame,
    _page_fault_error_code: PageFaultErrorCode,
) {
    let mut manager = PANIC_MANAGER.lock();
    let faulty_address = Cr2::read().unwrap();

    manager.bug_check(
        ErrorTypeEnum::PageFaultInNonpagedArea,
        Some((&faulty_address).as_ptr() as *const u32),
        None,
        None,
        None,
    );
}

extern "x86-interrupt" fn hypervisor_error(_stack_frame: InterruptStackFrame) {
    let mut manager = PANIC_MANAGER.lock();

    manager.bug_check(ErrorTypeEnum::HypervisorError, None, None, None, None);
}

lazy_static! {
    pub static ref InterruptTable: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error);
        idt.overflow.set_handler_fn(overflow);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.double_fault.set_handler_fn(double_fault);

        idt.breakpoint.set_handler_fn(breakpoint);
        idt.invalid_tss.set_handler_fn(invalid_tss);

        idt.page_fault.set_handler_fn(page_fault);

        idt.virtualization.set_handler_fn(hypervisor_error);

        idt
    };
}

pub fn load() {
    InterruptTable.load();
}
