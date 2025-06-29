use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str;
use crate::base::position::Position;
use std::hash::{Hash, Hasher};
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::a_move::MoveType::{Castling, EnPassant, Normal, PawnPromotion};
use crate::base::ambiguous_origin::OriginStatus;
use crate::figure::a_figure::FigureType;

// TODO MoveData should implement Claim as soon as it's added to the language.
// see https://smallcultfollowing.com/babysteps/blog/2024/06/21/claim-auto-and-otherwise/
#[derive(Debug, Copy, Clone)]
pub struct MoveData {
    pub given_from_to: FromTo,
    pub figure_moved: FigureType,
    pub figure_captured: Option<FigureType>,
    pub move_type: MoveType, // TODO: make this a Box<MoveType> or Rc<MoveType> together with a static lifetime instance of Rc/Box<MoveType::Normal>
    pub origin_status: OriginStatus,
    pub is_check: bool,
}

impl MoveData {
    pub fn new(
        given_move: FromTo,
        figure_moved: FigureType,
        figure_captured: Option<FigureType>,
        origin_status: OriginStatus,
        is_check: bool,
    ) -> MoveData {
        MoveData {
            given_from_to: given_move,
            figure_moved,
            figure_captured,
            move_type: Normal,
            origin_status,
            is_check,
        }
    }

    pub fn new_en_passant(
        given_move: FromTo,
        origin_status: OriginStatus,
        is_check: bool,
    ) -> MoveData {
        let captured_pawn_pos= Position::new_unchecked(given_move.to.column, given_move.from.row);
        MoveData {
            given_from_to: given_move,
            figure_moved: FigureType::Pawn,
            figure_captured: Some(FigureType::Pawn),
            move_type: EnPassant {captured_pawn_pos},
            origin_status,
            is_check,
        }
    }

    pub fn new_pawn_promotion(
        given_move: FromTo,
        figure_captured: Option<FigureType>,
        promotion_type: PromotionType,
        origin_status: OriginStatus,
        is_check: bool,
    ) -> MoveData {
        MoveData {
            given_from_to: given_move,
            figure_moved: FigureType::Pawn,
            figure_captured,
            move_type: PawnPromotion { promoted_to: promotion_type },
            origin_status,
            is_check,
        }
    }

    pub fn new_castling(
        given_move: FromTo,
        is_check: bool,
    ) -> MoveData {
        let king_from: Position = given_move.from;
        let rook_from: Position = given_move.to;
        let castling_row = king_from.row;
        let is_kingside_castling = king_from.column < rook_from.column;
        let (king_to, rook_to, castling_type) = if is_kingside_castling {
            (Position::new_unchecked(6, castling_row),
             Position::new_unchecked(5, castling_row),
             CastlingType::KingSide)
        } else {
            (Position::new_unchecked(2, castling_row),
             Position::new_unchecked(3, castling_row),
             CastlingType::QueenSide)
        };
        MoveData {
            given_from_to: given_move,
            figure_moved: FigureType::King,
            figure_captured: None,
            move_type: Castling {
                castling_type,
                king_move: FromTo::new(king_from, king_to),
                rook_move: FromTo::new(rook_from, rook_to),
            },
            origin_status: OriginStatus::Unambiguous,
            is_check,
        }
    }

    pub fn did_catch_figure(&self) -> bool {
        self.figure_captured.is_some()
    }

    pub fn is_pawn_move(&self) -> bool {
        self.figure_moved == FigureType::Pawn
    }

    // needed for fen computation
    pub fn did_make_progress(&self) -> bool {
        self.is_pawn_move() || self.did_catch_figure()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FromTo {
    pub from: Position,
    pub to: Position,
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for FromTo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize((self.from.index<< 6) + self.to.index);
    }
}

impl FromTo {
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
        }
    }

    pub fn from_code(code: &str) -> Self {
        code.parse::<FromTo>().unwrap_or_else(|_| panic!("illegal Move code: {}", code))
    }

    pub fn toggle_rows(&self) -> Self {
        Self {
            from: self.from.toggle_row(),
            to: self.to.toggle_row(),
        }
    }
}

impl str::FromStr for FromTo {
    type Err = ChessError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        Ok(FromTo {
            from: code[0..2].parse::<Position>()?,
            to: code[2..4].parse::<Position>()?,
        })
    }
}

impl Display for FromTo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}

impl Debug for FromTo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Move {
    pub from_to: FromTo,
    pub promotion_type: Option<PromotionType>,
}

impl Move {
    pub fn new(from_to: FromTo) -> Move {
        Move {
            from_to,
            promotion_type: None,
        }
    }

    pub fn new_with_promotion(from_to: FromTo, promotion_type: PromotionType) -> Move {
        Move {
            from_to,
            promotion_type: Some(promotion_type),
        }
    }



    pub fn toggle_rows(&self) -> Self {
        Self {
            from_to: self.from_to.toggle_rows(),
            promotion_type: self.promotion_type,
        }
    }
}

impl str::FromStr for Move {
    type Err = ChessError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        match code.len() {
            4 => {
                let from_to = code.parse::<FromTo>()?;
                Ok(Move::new(from_to))
            }
            5 => {
                let from_to = code[0..4].parse::<FromTo>()?;
                let pawn_move_type = code[4..5].parse::<PromotionType>()?;
                Ok(Move::new_with_promotion(from_to, pawn_move_type))
            }
            _ => {
                Err(ChessError {
                    msg: format!("illegal move format: {}", code),
                    kind: ErrorKind::IllegalFormat,
                })
            }
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.from_to)?;
        if let Some(promotion_type) = self.promotion_type {
            write!(f, "{}", promotion_type)?
        };
        Ok(())
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from_to.hash(state);
    }
}

// Default is needed, so that Move can be stored in a TinyVec
impl Default for Move {
    fn default() -> Self {
        Move::new(FromTo::new(
            // default values should never be used, so illegal values are fine
            // (they are necessary for TinyVec)
            Position::new_unchecked(9, 9),
            Position::new_unchecked(9, 9),
        ))
    }
}

pub fn toggle_rows(moves: &[Move]) -> Vec<Move> {
    moves.iter().map(|a_move| a_move.toggle_rows()).collect()
}

pub const EXPECTED_MAX_NUMBER_OF_MOVES: usize = 80;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PromotionType {
    Rook,
    Knight,
    Bishop,
    Queen,
}

impl PromotionType {
    pub fn get_figure_type(&self) -> FigureType {
        match self {
            PromotionType::Rook => {FigureType::Rook}
            PromotionType::Knight => {FigureType::Knight}
            PromotionType::Bishop => {FigureType::Bishop}
            PromotionType::Queen => {FigureType::Queen}
        }
    }

    pub fn as_encoded(&self) -> char {
        match self {
            PromotionType::Rook => 'R',
            PromotionType::Knight => 'N',
            PromotionType::Bishop => 'B',
            PromotionType::Queen => 'Q'
        }
    }
}

impl str::FromStr for PromotionType {
    type Err = ChessError;

    fn from_str(s: &str) -> Result<PromotionType, Self::Err> {
        match s {
            "Q" => Ok(PromotionType::Queen),
            "R" => Ok(PromotionType::Rook),
            "N" => Ok(PromotionType::Knight),
            "B" => Ok(PromotionType::Bishop),
            _ => Err(ChessError{
                msg: format!("unknown pawn promotion type: {}. Only 'QRNB' are allowed.", s),
                kind: ErrorKind::IllegalFormat
            }),
        }
    }
}

impl Display for PromotionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_encoded())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CastlingType {
    KingSide,
    QueenSide,
}

impl Display for MoveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = match self {
            Normal => {"-"}
            PawnPromotion { promoted_to } => {
                match promoted_to {
                    PromotionType::Rook => {"R"}
                    PromotionType::Knight => {"N"}
                    PromotionType::Bishop => {"B"}
                    PromotionType::Queen => {"Q"}
                }
            }
            EnPassant { .. } => {"e"}
            Castling { castling_type: c_type, .. } => {match c_type {
                CastlingType::KingSide => {"c"}
                CastlingType::QueenSide => {"C"}
            }}
        };
        write!(f, "{}", code)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveType {
    Normal,
    PawnPromotion{ promoted_to: PromotionType },
    EnPassant { captured_pawn_pos: Position },
    Castling { castling_type: CastlingType, king_move: FromTo, rook_move: FromTo }
}


//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::base::a_move::{FromTo, Move, PromotionType};
    use crate::base::position::Position;

    #[rstest(
        from_to, from, to,
        case("b1c3", "b1", "c3"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_fromto_from_str(
        from_to: FromTo,
        from: Position,
        to: Position,
    ) {
        assert_eq!(from, from_to.from);
        assert_eq!(to, from_to.to);
    }


    #[rstest(
        a_move, from, to, promotes_to,
        case("b1c3", "b1", "c3", None),
        case("g7f8Q", "g7", "f8", Some("Q")),
        case("a1a8R", "a1", "a8", Some("R")),
        case("a4h4N", "a4", "h4", Some("N")),
        case("b8h2B", "b8", "h2", Some("B")),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_move_from_str(
        a_move: Move,
        from: Position,
        to: Position,
        promotes_to: Option<&'static str>,
    ) {
        assert_eq!(from, a_move.from_to.from);
        assert_eq!(to, a_move.from_to.to);
        let given_promotion_type: Option<PromotionType> = promotes_to.map(|it| it.parse().expect(format!("unknown PromotionType: {it}").as_str()));
        assert_eq!(given_promotion_type, a_move.promotion_type);
    }

    #[rstest(
        given_promotion_type,
        case("R"),
        case("N"),
        case("B"),
        case("Q"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_promotion_type_traits_display_and_fromstr_work_together(
        given_promotion_type: PromotionType,
    ) {
        let type_str = format!("{given_promotion_type}");
        let actual_promotion_type: PromotionType = type_str.as_str().parse().unwrap();
        assert_eq!(actual_promotion_type, given_promotion_type);
    }

    #[test]
    fn test_knight_encodes_as_n() {
        assert_eq!('N', PromotionType::Knight.as_encoded());
    }
}