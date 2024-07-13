use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Disallowable {
    value: bool,
}

impl Disallowable {
    pub fn new(value: bool) -> Disallowable {
        Disallowable {
            value,
        }
    }

    pub fn disallow(&mut self) {
        self.value = false;
    }

    pub fn is_still_allowed(&self) -> bool {
        self.value
    }
}

impl fmt::Display for Disallowable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
