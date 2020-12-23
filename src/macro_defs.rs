#[macro_export]
macro_rules! lisp_error {
    ( $fmt:expr $(, $arg:expr ),* ) => {
        use crate::error::LispError;
        return Err(LispError::new(format!($fmt, $( $arg )*)));
    };
}
