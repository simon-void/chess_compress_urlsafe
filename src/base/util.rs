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

#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::Display;

    pub(crate) fn vec_to_str<A: Display>(vec: &Vec<A>, separator: &str) -> String {
        format!("[{}]", vec.iter().map(|pos|format!("{pos}")).collect::<Vec<String>>().join(separator))
    }
}