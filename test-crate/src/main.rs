use std::ffi::c_char;

extern "C" {
    fn greet(name: *const c_char);
}

fn main() {
    unsafe {
        std::env::args().skip(1).for_each(|arg| {
            greet(arg.as_ptr() as *const c_char);
        });
    }
}
