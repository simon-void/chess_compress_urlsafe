pub use crate::base::a_move::*;
pub use crate::base::errors::*;
pub use crate::base::position::*;
pub use crate::base::direction::*;
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