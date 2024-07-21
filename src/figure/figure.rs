use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use FigureType::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::base::color::Color;
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Figure {
    pub fig_type: FigureType,
    pub color: Color,
}

impl Figure {
    pub fn get_fen_char(&self) -> char {
        match self.fig_type {
            Pawn => {if self.color == Color::White {'P'} else {'p'}}
            Rook => {if self.color == Color::White {'R'} else {'r'}}
            Knight => {if self.color == Color::White {'N'} else {'n'}}
            Bishop => {if self.color == Color::White {'B'} else {'b'}}
            Queen => {if self.color == Color::White {'Q'} else {'q'}}
            King => {if self.color == Color::White {'K'} else {'k'}}
        }
    }
}

impl FromStr for Figure {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        match desc {
            "♙" => Ok(Figure{fig_type: Pawn, color: Color::White}),
            "♟" => Ok(Figure{fig_type: Pawn, color: Color::Black}),
            "♖" => Ok(Figure{fig_type: Rook, color: Color::White}),
            "♜" => Ok(Figure{fig_type: Rook, color: Color::Black}),
            "♘" => Ok(Figure { fig_type: Knight, color: Color::White }),
            "♞" => Ok(Figure { fig_type: Knight, color: Color::Black }),
            "♗" => Ok(Figure { fig_type: Bishop, color: Color::White }),
            "♝" => Ok(Figure { fig_type: Bishop, color: Color::Black }),
            "♕" => Ok(Figure { fig_type: Queen, color: Color::White }),
            "♛" => Ok(Figure { fig_type: Queen, color: Color::Black }),
            "♔" => Ok(Figure { fig_type: King, color: Color::White }),
            "♚" => Ok(Figure { fig_type: King, color: Color::Black }),
            _ => Err(ChessError{
                msg: format!("unexpected character, utf-chess symbol like ♙ expected but got {}", desc),
                kind: ErrorKind::IllegalFormat,
            })
        }
    }
}

impl Display for Figure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.fig_type {
            Pawn => {if self.color==Color::White {"♙"} else {"♟"}}
            Rook => {if self.color==Color::White {"♖"} else {"♜"}}
            Knight => {if self.color==Color::White {"♘"} else {"♞"}}
            Bishop => {if self.color==Color::White {"♗"} else {"♝"}}
            Queen => {if self.color==Color::White {"♕"} else {"♛"}}
            King => {if self.color==Color::White {"♔"} else {"♚"}}
        };
        write!(f,"{}", symbol)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FigureAndPosition {
    pub figure: Figure,
    pub pos: Position,
}

impl FromStr for FigureAndPosition {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FigureType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Display for FigureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Pawn => 'P',
            Rook => 'R',
            Knight => 'N',
            Bishop => 'B',
            Queen => 'Q',
            King => 'K',
        };
        write!(f,"{}", symbol)
    }
}

impl FromStr for FigureType {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        match desc {
            "P" => Ok(Pawn),
            "R" => Ok(Rook),
            "N" => Ok(Knight),
            "B" => Ok(Bishop),
            "Q" => Ok(Queen),
            "K" => Ok(King),
            _ => Err(ChessError{
                msg: format!("unexpected character, char P, R, N, B, Q, or K expected but got {}", desc),
                kind: ErrorKind::IllegalFormat,
            })
        }
    }
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::figure::figure::FigureType;

    #[rstest(
        given_figure_type,
        case("P"),
        case("R"),
        case("N"),
        case("B"),
        case("Q"),
        case("K"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_FigureType_Display_and_FromStr(
        given_figure_type: FigureType,
    ) {
        let type_str = format!("{given_figure_type}");
        let actual_figure_type: FigureType = type_str.as_str().parse().unwrap();
        assert_eq!(actual_figure_type, given_figure_type);
    }
}
