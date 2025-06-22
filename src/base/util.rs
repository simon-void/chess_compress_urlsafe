use std::fmt;
use std::fmt::Display;

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

impl Display for Disallowable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn vec_to_str<A: Display>(vec: &[A], separator: &str) -> String {
    format!("[{}]", vec.iter().map(|pos|format!("{pos}")).collect::<Vec<String>>().join(separator))
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;
    use std::fmt::Display;
    use std::hash::Hash;
    use std::str::FromStr;
    use itertools::Itertools;
    use crate::base::errors::ChessError;
    use crate::base::errors::ErrorKind::IllegalConfig;

    pub fn vec_has_uniquely_same_elements_as_set<A: Eq>(vec: &Vec<A>, set: &HashSet<A>) -> bool {
        if vec.len() != set.len() {
            return false;
        };

        vec.iter().all(|it|set.iter().contains(it))
    }

    pub fn parse_to_vec<A: FromStr<Err=ChessError>>(str: &str, separator: &str) -> Result<Vec<A>, ChessError> {
        if separator.is_empty() {
            return Err(ChessError{
                msg: "separator mus not be empty".to_string(),
                kind: IllegalConfig
            })
        }
        str.split(separator).map(str::trim).filter(|it| !it.is_empty()).map(|it|{
            it.parse::<A>()
        }).collect()
    }

    pub fn vec_into_set<A: Copy + Hash + Eq>(vec: &Vec<A>) -> HashSet<A> {
        vec.iter().map(|it| *it).collect()
    }

    pub fn parse_to_set<A: FromStr<Err=ChessError> + Hash + Eq>(str: &str, separator: &str) -> Result<HashSet<A>, ChessError> {
        if separator.is_empty() {
            return Err(ChessError{
                msg: "separator mus not be empty".to_string(),
                kind: IllegalConfig
            })
        }
        str.split(separator).map(str::trim).filter(|it| !it.is_empty()).map(|it| {
            it.parse::<A>()
        }).collect()
    }

    pub fn set_to_str<A: Display>(vec: &HashSet<A>, separator: &str) -> String {
        format!("[{}]", vec.iter().map(|pos|format!("{pos}")).collect::<Vec<String>>().join(separator))
    }
}