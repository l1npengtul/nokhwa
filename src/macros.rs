/// Converts $from into $to
/// Example usage:
/// `tryinto_num(i32, a_unsigned_32_bit_num)`
/// Designed to deal with infallible. If not, it should be manually handled.
/// # Errors
/// If fails to convert(note: should not happen) then you messed up.
#[macro_export]
macro_rules! tryinto_num {
    ($to:ty, $from:expr) => {{
        use std::convert::TryFrom;
        match <$to>::try_from($from) {
            Ok(v) => v,
            Err(why) => {
                return Err(crate::NokhwaError::GeneralError(format!(
                    "Failed to convert {}, {}",
                    $from,
                    why.to_string()
                )))
            }
        }
    }};
}
