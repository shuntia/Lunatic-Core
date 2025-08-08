/// Loads symbol, lib:Library, sym: The name of the fn
#[macro_export]
macro_rules! load_symbol_c {
    // If no output type specified, assume ()
    ($lib:expr, $sym:literal, ($($in:ty),*)) => {
        unsafe {
            match $lib.get::<unsafe extern "C" fn($($in),*) -> ()>($sym.as_bytes()) {
                Ok(s) => Some(*s),
                Err(_) => None,
            }
        }
    };
    // If output type specified
    ($lib:expr, $sym:literal, ($($in:ty),*) -> $out:ty) => {
        unsafe {
            match $lib.get::<unsafe extern "C" fn($($in),*) -> $out>($sym.as_bytes()) {
                Ok(s) => Some(*s),
                Err(_) => None,
            }
        }
    };
    // Original fallback without specifying types, assume fn() -> ()
    ($lib:expr, $sym:literal) => {
        unsafe {
            match $lib.get::<unsafe extern "C" fn() -> ()>($sym.as_bytes()) {
                Ok(s) => Some(*s),
                Err(_) => None,
            }
        }
    };
}
