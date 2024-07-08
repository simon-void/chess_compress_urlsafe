mod a_move;
pub(crate) mod direction;
mod errors;
mod position;

pub use a_move::*;
pub use errors::*;
pub use position::*;
pub use direction::*;
use std::fmt;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum Color {
    Black, White,
}

impl Color {
    pub fn toggle(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn get_fen_char(&self) -> char {
        match self {
            Color::Black => {'b'}
            Color::White => {'w'}
        }
    }

    pub fn get_ground_row(&self) -> i8 {
        match self {
            Color::Black => {7}
            Color::White => {0}
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

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
