#![no_std]

unsafe extern "C" {
    fn zig_add(left: i32, right: i32) -> i32;
}

/// Calls through Rust into the Zig-compiled static library.
#[no_mangle]
pub extern "C" fn rust_calls_zig(left: i32, right: i32) -> i32 {
    unsafe { zig_add(left, right) * 2 }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
