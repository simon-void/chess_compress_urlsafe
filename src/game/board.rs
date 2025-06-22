use std::fmt::{Display, Formatter, Result};
use std::ops::Range;
use crate::base::color::Color;
use crate::base::direction::Direction;
use crate::base::position::{I8_RANGE_07, Position};
use crate::figure::a_figure::{Figure, FigureType};

static WHITE_PAWN: Figure = Figure {fig_type:FigureType::Pawn, color: Color::White,};
static WHITE_QUEEN_SIDE_ROOK: Figure = Figure {fig_type:FigureType::Rook, color: Color::White,};
static WHITE_KING_SIDE_ROOK: Figure = Figure {fig_type:FigureType::Rook, color: Color::White,};
static WHITE_KNIGHT: Figure = Figure {fig_type:FigureType::Knight, color: Color::White,};
static WHITE_BISHOP: Figure = Figure {fig_type:FigureType::Bishop, color: Color::White,};
static WHITE_QUEEN: Figure = Figure {fig_type:FigureType::Queen, color: Color::White,};
static WHITE_KING: Figure = Figure {fig_type:FigureType::King, color: Color::White,};

static BLACK_PAWN: Figure = Figure {fig_type:FigureType::Pawn, color: Color::Black,};
static BLACK_QUEEN_SIDE_ROOK: Figure = Figure {fig_type:FigureType::Rook, color: Color::Black,};
static BLACK_KING_SIDE_ROOK: Figure = Figure {fig_type:FigureType::Rook, color: Color::Black,};
static BLACK_KNIGHT: Figure = Figure {fig_type:FigureType::Knight, color: Color::Black,};
static BLACK_BISHOP: Figure = Figure {fig_type:FigureType::Bishop, color: Color::Black,};
static BLACK_QUEEN: Figure = Figure {fig_type:FigureType::Queen, color: Color::Black,};
static BLACK_KING: Figure = Figure {fig_type:FigureType::King, color: Color::Black,};


pub type FiguresWithPosArray = [Option<(FigureType, Position)>; 16];

#[derive(Clone, Debug)]
pub struct Board {
    state: [Option<Figure>; 64],
    number_of_figures: isize,
}

impl Board {
    pub fn classic() -> Board {
        Board {
            number_of_figures: 32,
            state: [
                Some(WHITE_QUEEN_SIDE_ROOK),
                Some(WHITE_KNIGHT),
                Some(WHITE_BISHOP),
                Some(WHITE_QUEEN),
                Some(WHITE_KING),
                Some(WHITE_BISHOP),
                Some(WHITE_KNIGHT),
                Some(WHITE_KING_SIDE_ROOK),
                Some(WHITE_PAWN), Some(WHITE_PAWN), Some(WHITE_PAWN), Some(WHITE_PAWN),
                Some(WHITE_PAWN), Some(WHITE_PAWN), Some(WHITE_PAWN), Some(WHITE_PAWN),
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                Some(BLACK_PAWN), Some(BLACK_PAWN), Some(BLACK_PAWN), Some(BLACK_PAWN),
                Some(BLACK_PAWN), Some(BLACK_PAWN), Some(BLACK_PAWN), Some(BLACK_PAWN),
                Some(BLACK_QUEEN_SIDE_ROOK),
                Some(BLACK_KNIGHT),
                Some(BLACK_BISHOP),
                Some(BLACK_QUEEN),
                Some(BLACK_KING),
                Some(BLACK_BISHOP),
                Some(BLACK_KNIGHT),
                Some(BLACK_KING_SIDE_ROOK),
            ],
        }
    }

    pub fn empty() -> Board {
        Board {
            number_of_figures: 0,
            state: [None; 64],
        }
    }

    pub fn get_all_figures_of_color(&self, color: Color) -> [Option<(Figure, Position)>; 16] {
        let mut figures: [Option<(Figure, Position)>; 16] = [None; 16];
        let mut next_index: usize = 0;
        for state_index in USIZE_RANGE_063 {
            if let Some(figure) = self.state[state_index] {
                if figure.color == color {
                    figures[next_index] = Some(
                        (figure, Position::from_index_unchecked(state_index))
                    );
                    next_index += 1;
                }
            }
        }
        figures
    }

    pub fn get_white_and_black_figures(&self) -> (FiguresWithPosArray, FiguresWithPosArray) {
        let mut white_figures: FiguresWithPosArray = [None; 16];
        let mut black_figures: FiguresWithPosArray = [None; 16];
        let mut next_white_index: usize = 0;
        let mut next_black_index: usize = 0;

        for state_index in USIZE_RANGE_063 {
            if let Some(figure) = self.state[state_index] {
                if figure.color == Color::White {
                    white_figures[next_white_index] = Some(
                        (figure.fig_type, Position::from_index_unchecked(state_index))
                    );
                    next_white_index += 1;
                } else {
                    black_figures[next_black_index] = Some(
                        (figure.fig_type, Position::from_index_unchecked(state_index))
                    );
                    next_black_index += 1;
                }
            }
        }
        (white_figures, black_figures)
    }

    pub fn get_figure(&self, pos: Position) -> Option<Figure> {
        self.state[pos.index]
    }

    /**
    * returns if a figure was caught/replaced on that position
    */
    pub fn set_figure(&mut self, pos: Position, figure: Figure) -> CaptureInfoOption {
        let old_content = self.state[pos.index];
        self.state[pos.index] = Some(figure);

        if let Some(old_figure) = old_content {
            CaptureInfoOption::from_some(old_figure, pos)
        } else {
            self.number_of_figures += 1;
            CaptureInfoOption::from_none()
        }
    }

    pub fn clear_field(&mut self, pos: Position) {
        self.number_of_figures -= 1;
        self.state[pos.index] = None;
    }

    pub fn contains_sufficient_material_to_continue(&self) -> bool {
        if self.number_of_figures > 6 {
            return true;
        }

        let mut white_knight_nr = 0;
        let mut found_white_bishop = false;
        let mut black_knight_nr = 0;
        let mut found_black_bishop = false;

        for state_index in USIZE_RANGE_063 {
            if let Some(figure) = self.state[state_index] {
                match figure.fig_type {
                    FigureType::Pawn | FigureType::Rook | FigureType::Queen => {return true;}
                    FigureType::Knight => {
                        match figure.color {
                            Color::Black => { black_knight_nr += 1; }
                            Color::White => { white_knight_nr += 1; }
                        }
                    }
                    FigureType::Bishop => {
                        match figure.color {
                            Color::Black => {
                                // this is basically a black_bishop_nr == 2 check
                                if found_black_bishop {
                                    return true;
                                }
                                found_black_bishop = true;
                            }
                            Color::White => {
                                // this is basically a black_bishop_nr == 2 check
                                if found_white_bishop {
                                    return true;
                                }
                                found_white_bishop = true;
                            }
                        }
                    }
                    FigureType::King => {}
                }
            }
        }

        (found_white_bishop && white_knight_nr != 0) ||
            (found_black_bishop && black_knight_nr != 0) ||
            (white_knight_nr>2) || (black_knight_nr>2)
    }

    pub fn is_empty(&self, pos: Position) -> bool {
        self.get_figure(pos).is_none()
    }

    pub fn are_intermediate_pos_free(&self, from_pos: Position, from2to_direction: Direction, to_pos: Position) -> bool {
        let mut pos = from_pos;
        loop {
            pos = pos.step(from2to_direction).expect("sequence should terminate with to_pos");
            if pos == to_pos {
                return true;
            }
            if self.get_figure(pos).is_some() {
                return false;
            }
        }
    }

    pub fn contains_figure(&self, pos: Position, fig_type: FigureType, color: Color) -> bool {
        match self.state[pos.index] {
            None => false,
            Some(figure) => {
                figure.fig_type == fig_type && figure.color == color
            }
        }
    }

    pub fn contains_color(&self, pos: Position, color: Color) -> bool {
        match self.state[pos.index] {
            None => false,
            Some(figure) => figure.color == color
        }
    }

    pub fn get_content_type(&self, pos: Position, color: Color) -> FieldContent {
        match self.get_figure(pos) {
            Some(figure) => if figure.color==color {
                FieldContent::OwnFigure
            } else {
                FieldContent::OpponentFigure
            },
            None => FieldContent::Empty,
        }
    }

    pub fn get_fen_part1(&self) -> String {
        let mut fen_part1 = String::with_capacity(72);
        let mut index_range_end: usize = 64;
        loop {
            let mut fields_without_figure: usize = 0;
            for pos_index in index_range_end-8..index_range_end {
                match self.state[pos_index] {
                    None => {fields_without_figure+=1;}
                    Some(figure) => {
                        if fields_without_figure != 0 {
                            fen_part1.push_str(fields_without_figure.to_string().as_str());
                            fields_without_figure = 0;
                        }
                        fen_part1.push(figure.get_fen_char());
                    }
                }
            }
            if fields_without_figure != 0 {
                fen_part1.push_str(fields_without_figure.to_string().as_str());
            }
            if index_range_end == 8 {
                break;
            } else {
                fen_part1.push('/');
                index_range_end -= 8;
            }
        }
        fen_part1
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f)?;
        for row_index in I8_RANGE_07.rev() {
            for column_index in I8_RANGE_07 {
                let figure_index = Position::new_unchecked(column_index, row_index).index;
                let fig_option = self.state[figure_index];
                match fig_option {
                    None => {write!(f, "_")},
                    Some(figure) => {write!(f, "{}", figure)},
                }?;
            }
            writeln!(f, " {}", row_index + 1)?;
        }
        writeln!(f, "abcdefgh")
    }
}

pub const USIZE_RANGE_063: Range<usize> = 0..64;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FieldContent {
    Empty, OwnFigure, OpponentFigure,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CaptureInfoOption(
    Option<(Figure, Position)>
);

impl CaptureInfoOption {
    pub fn from_some(figure_caught: Figure, figure_caught_on: Position) -> CaptureInfoOption {
        CaptureInfoOption(Some((figure_caught, figure_caught_on)))
    }

    pub fn from_none() -> CaptureInfoOption {
        CaptureInfoOption(None)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn get_captured_figure(&self) -> Option<Figure> {
        self.0.map(|capture_info|{capture_info.0})
    }

    pub fn get_captured_on_pos(&self) -> Option<Position> {
        self.0.map(|capture_info|{capture_info.1})
    }

    pub fn get_captured_figure_type(&self) -> Option<FigureType> {
        self.0.map(|capture_info|{capture_info.0.fig_type})
    }
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::game::game_state::GameState;
    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        game_state, expected_fen_part1,
        case("", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"),
        case("e2e4", "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR"),
        case("b1a3 g8h6 e2e4", "rnbqkb1r/pppppppp/7n/8/4P3/N7/PPPP1PPP/R1BQKBNR"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_get_fen_part1(
        game_state: GameState,
        expected_fen_part1: &str,
    ) {
        let actual_fen_part1 = game_state.board.get_fen_part1();
        assert_eq!(actual_fen_part1, String::from(expected_fen_part1));
    }

    #[rstest(
        game_state, expected_nr_of_figures,
        case("e2e4", 32),
        case("e2e4 d7d5 e4d5", 31), // capture
        case("a2a4 h7h6 a4a5 b7b5 a5b6", 31), // capture en passant
        case("a2a4 h7h6 a4a5 b7b5 a5b6 h6h5 b6b7 b8c6 b7b8Q", 31), // pawn promotion without capture
        case("a2a4 h7h6 a4a5 b7b5 a5b6 h6h5 b6b7 b8c6 b7a8Q", 30), // pawn promotion with capture
        case("g2g3 a7a6 f1g2 a6a5 g1f3 a5a4 e1h1", 32),             // short castling
        case("d2d3 a7a6 c1f4 a6a5 d1d2 a5a4 b1c3 a4a3 e1a1", 32), // long castling
        case("white ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", 6),
        case("black ♖a1 ♔e1 ♚e8", 3),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_number_of_figures(
        game_state: GameState,
        expected_nr_of_figures: isize,
    ) {
        let actual_nr_of_figures = game_state.board.number_of_figures;
        assert_eq!(actual_nr_of_figures, expected_nr_of_figures);
    }
}
