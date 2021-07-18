#[derive(Debug, Clone, Copy)]
pub struct LineInfo {
    pub line: i32,
    pub col: i32,
}

impl LineInfo {
    pub fn new(line: i32, col: i32) -> Self {
        Self { line, col }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    MathError,
    TypeError,
    SyntaxError,
    RuntimeError,
}

#[derive(Clone, Debug)]
pub struct Error {
    info: LineInfo,
    error_type: ErrorType,
    error: String,
}

impl Error {
    pub fn new(info: LineInfo, error: String, error_type: ErrorType) -> Self {
        Self { info, error, error_type }
    }

    pub fn display(&self) {
        eprintln!(
            "[{}:{}] {:?}: {}",
            self.info.line,
            self.info.col,
            self.error_type,
            self.error,
        );
    }
}
