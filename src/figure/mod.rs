pub mod functions;

use std::fmt;
use crate::base::*;
use std::str;
use serde::Serialize;
use crate::game::GameState;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub struct Figure {
    pub fig_type: FigureType,
    pub color: Color,
}

impl Figure {
    pub fn for_reachable_moves(&self, _pos: Position, _match_state: &GameState, _move_collector: &mut Moves) {
        // functions::is_reachable_by::for_reachable_moves(self.fig_type, pos, match_state, move_collector)
        panic!("TODO: reimplement or delete")
    }

    pub fn get_fen_char(&self) -> char {
        match self.fig_type {
            FigureType::Pawn => {if self.color == Color::White {'P'} else {'p'}}
            FigureType::Rook => {if self.color == Color::White {'R'} else {'r'}}
            FigureType::Knight => {if self.color == Color::White {'N'} else {'n'}}
            FigureType::Bishop => {if self.color == Color::White {'B'} else {'b'}}
            FigureType::Queen => {if self.color == Color::White {'Q'} else {'q'}}
            FigureType::King => {if self.color == Color::White {'K'} else {'k'}}
        }
    }
}

impl str::FromStr for Figure {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        match desc {
            "♙" => Ok(Figure{fig_type: FigureType::Pawn, color: Color::White}),
            "♟" => Ok(Figure{fig_type: FigureType::Pawn, color: Color::Black}),
            "♖" => Ok(Figure{fig_type: FigureType::Rook, color: Color::White}),
            "♜" => Ok(Figure{fig_type: FigureType::Rook, color: Color::Black}),
            "♘" => Ok(Figure { fig_type: FigureType::Knight, color: Color::White }),
            "♞" => Ok(Figure { fig_type: FigureType::Knight, color: Color::Black }),
            "♗" => Ok(Figure { fig_type: FigureType::Bishop, color: Color::White }),
            "♝" => Ok(Figure { fig_type: FigureType::Bishop, color: Color::Black }),
            "♕" => Ok(Figure { fig_type: FigureType::Queen, color: Color::White }),
            "♛" => Ok(Figure { fig_type: FigureType::Queen, color: Color::Black }),
            "♔" => Ok(Figure { fig_type: FigureType::King, color: Color::White }),
            "♚" => Ok(Figure { fig_type: FigureType::King, color: Color::Black }),
            _ => Err(ChessError{
                msg: format!("unexpected character, utf-chess symbol like ♙ expected but got {}", desc),
                kind: ErrorKind::IllegalFormat,
            })
        }
    }
}

impl fmt::Display for Figure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.fig_type {
            FigureType::Pawn => {if self.color==Color::White {"♙"} else {"♟"}}
            FigureType::Rook => {if self.color==Color::White {"♖"} else {"♜"}}
            FigureType::Knight => {if self.color==Color::White {"♘"} else {"♞"}}
            FigureType::Bishop => {if self.color==Color::White {"♗"} else {"♝"}}
            FigureType::Queen => {if self.color==Color::White {"♕"} else {"♛"}}
            FigureType::King => {if self.color==Color::White {"♔"} else {"♚"}}
        };
        write!(f,"{}", symbol)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FigureAndPosition {
    pub figure: Figure,
    pub pos: Position,
}

impl str::FromStr for FigureAndPosition {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        let split_point = desc.len()-2; // splitting is a bit more complicated since utf-8 chars like ♔ take more space than 1 byte
        let figure = desc[..split_point].parse::<Figure>()?;
        let pos = desc[split_point..].parse::<Position>()?;

        Ok(FigureAndPosition{
            figure,
            pos,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum FigureType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}
