use std::fmt::*;

#[derive(Debug)]
pub struct ChessError {
    pub msg: String,
    pub kind: ErrorKind,
}

impl Display for ChessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{:?}: {}", self.kind, self.msg)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IllegalConfig,
    IllegalFormat,
    IllegalMove,
}