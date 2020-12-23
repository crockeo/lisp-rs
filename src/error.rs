pub struct LispError {
    message: String,
    // TODO: this is currently unused, but should be used Soon™ so that you can figure out where
    // errors are.
    _row: usize,
    _col: usize,
}

impl LispError {
    pub fn new<S: AsRef<str>>(message: S) -> LispError {
        LispError {
            message: message.as_ref().to_string(),
            _row: 0,
            _col: 0,
        }
    }

    pub fn message<'a>(&'a self) -> &'a str {
        self.message.as_str()
    }
}

pub type LispResult<T> = Result<T, LispError>;
