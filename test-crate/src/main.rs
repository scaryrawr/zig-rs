use std::ffi::{c_char, CStr};

extern "C" {
    fn greet(name: *const c_char) -> *const c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_greet() {
        let name = CString::new("World").expect("CString::new failed");
        unsafe {
            let result_ptr = greet(name.as_ptr());
            assert!(!result_ptr.is_null());

            let result = CStr::from_ptr(result_ptr);
            assert_eq!(
                result.to_str().expect("CString::to_str failed"),
                "Hello, World!"
            );
        }
    }
}

fn main() {
    unsafe {
        std::env::args().skip(1).for_each(|arg| {
            let message = greet(arg.as_ptr() as *const c_char);
            println!("{}", CStr::from_ptr(message).to_str().unwrap());
        });
    }
}
