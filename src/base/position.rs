use std::cmp::max;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::iter::{Iterator};
use std::ops::Range;
use std::str;
use crate::base::color::Color;
use crate::base::direction::Direction;
use crate::base::errors::{ChessError, ErrorKind};
use crate::game::board::{Board, FieldContent, USIZE_RANGE_063};

#[derive(Copy, Clone, Eq)]
pub struct Position {
    pub index: usize,
    pub column: i8,
    pub row: i8,
}

impl Position {
    pub fn new_checked(column: i8, row: i8) -> Option<Position> {
        if !(I8_RANGE_07.contains(&column) && I8_RANGE_07.contains(&row)) {
            return None
        }
        Some(Position::new_unchecked(column, row))
    }

    pub const fn new_unchecked(column: i8, row: i8) -> Position {
        // debug_assert!(
        //     I8_RANGE_07.contains(&column) && I8_RANGE_07.contains(&row),
        //     "column and row were expected to be 0..64 but were column: {} and row: {}",
        //     column, row
        // );
        Position {
            index: ((row*8)+column) as usize,
            column,
            row,
        }
    }

    pub fn from_index_unchecked(index: usize) -> Position {
        debug_assert!(
            USIZE_RANGE_063.contains(&index),
            "index was expected to be 0..64 but was {}",
            index
        );
        let i = index as i8;
        let column = i % 8;
        let row = i/8;
        debug_assert!(
          I8_RANGE_07.contains(&column) && I8_RANGE_07.contains(&row),
          "column and row were expected to be 0..64 but were column: {} and row: {}",
          column, row
        );

        Position {
            index: ((row*8)+column) as usize,
            column,
            row,
        }
    }

    pub fn from_code(code: &str) -> Position {
        code.parse::<Position>().unwrap_or_else(|_| panic!("illegal Position code: {}", code))
    }
    
    pub fn column_char(&self) -> char {
        (self.column + 97) as u8 as char
    }

    pub fn row_char(&self) -> char {
        (self.row + 49) as u8 as char
    }

    pub fn get_row_distance(&self, other: Position) -> i8 {
        (self.row - other.row).abs()
    }

    pub fn touches(&self, other: Position) -> bool {
        max((self.row - other.row).abs(), (self.column - other.column).abs()) == 1
    }

    pub fn step(&self, direction: Direction) -> Option<Position> {
        match direction {
            Direction::Right => {
                let new_column = self.column + 1;
                if new_column == 8 { None } else { Some(Position::new_unchecked(new_column, self.row)) }
            },
            Direction::Left => {
                let new_column = self.column - 1;
                if new_column == -1 { None } else { Some(Position::new_unchecked(new_column, self.row)) }
            },
            Direction::Up => {
                let new_row = self.row + 1;
                if new_row == 8 { None } else { Some(Position::new_unchecked(self.column, new_row)) }
            },
            Direction::Down => {
                let new_row = self.row - 1;
                if new_row == -1 { None } else { Some(Position::new_unchecked(self.column, new_row )) }
            },
            Direction::UpRight => Position::new_checked(self.column + 1, self.row + 1),
            Direction::UpLeft => Position::new_checked(self.column - 1, self.row + 1),
            Direction::DownLeft => Position::new_checked(self.column - 1, self.row - 1),
            Direction::DownRight => Position::new_checked(self.column + 1, self.row - 1),
        }
    }

    pub fn step_unchecked(&self, direction: Direction) -> Position {
        self.step(direction).unwrap()
    }

    fn jump(
        &self,
        column_delta: i8,
        row_delta: i8,
    ) -> Option<Position> {
        Position::new_checked(self.column + column_delta, self.row + row_delta)
    }

    pub fn count_reachable_directed_positions(
        &self,
        fig_color: Color,
        direction: Direction,
        board: &Board,
    ) -> usize {
        let mut last_pos = *self;
        let mut counter: usize = 0;
        loop {
            let new_pos = match last_pos.step(direction) {
                None => {break;}
                Some(pos) => {pos}
            };
            match board.get_figure(new_pos) {
                None => {counter += 1;}
                Some(figure) => {
                    if figure.color!=fig_color {
                        counter += 1;
                    }
                    break;
                }
            }
            last_pos = new_pos;
        }
        counter
    }

    pub fn count_reachable_knight_positions(
        &self,
        fig_color: Color,
        board: &Board,
    ) -> usize {
        self.reachable_knight_positions(fig_color, board).count()
    }

    pub fn reachable_directed_positions<'b>(
        &self,
        fig_color: Color,
        direction: Direction,
        board: &'b Board,
    ) -> DirectedPosIterator<'b > {
        DirectedPosIterator::new(*self, fig_color, direction, board)
    }

    pub fn reachable_knight_positions<'b>(
        &self,
        knight_color: Color,
        board: &'b Board,
    ) -> KnightPosIterator<'b> {
        KnightPosIterator::new(*self, knight_color, board)
    }

    pub fn is_on_ground_row(&self, color: Color) -> bool {
        match color {
            Color::Black if self.row == 7 => true,
            Color::White if self.row == 0 => true,
            _ => false,
        }
    }

    pub fn is_reachable_by_knight(&self, pos: Position) -> bool {
        let column_diff = (self.column - pos.column).abs();
        let row_diff = (self.row - pos.row).abs();
        column_diff != 0 && row_diff != 0 && (column_diff + row_diff) == 3
    }

    pub fn toggle_row(&self) -> Position {
        Position::new_unchecked(
            self.column, 7-self.row,
        )
    }

    pub fn get_direction(&self, to: Position) -> Option<Direction> {
        if *self == to {
           return None;
        }
        let row_diff = to.row - self.row;
        let column_diff = to.column - self.column;
        if row_diff == 0 {
            return if column_diff.is_positive() {
                Some(Direction::Right)
            } else {
                Some(Direction::Left)
            }
        }
        if column_diff == 0 {
            return if row_diff.is_positive() {
                Some(Direction::Up)
            } else {
                Some(Direction::Down)
            }
        }
        if row_diff.abs() != column_diff.abs() {
            return None;
        }
        if row_diff.is_positive() {
            if column_diff.is_positive() {
                Some(Direction::UpRight)
            } else {
                Some(Direction::UpLeft)
            }
        } else if column_diff.is_positive() {
            Some(Direction::DownRight)
        } else {
            Some(Direction::DownLeft)
        }
    }
}

impl str::FromStr for Position {
    type Err = ChessError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        let mut char_iter = code.chars();
        if code.len()!=2 {
            return Err(ChessError{
                msg: format!("Position str: {code} should consist of 2 chars not {}", code.len()),
                kind: ErrorKind::IllegalFormat
            });
        }

        fn get_index(maybe_char: Option<char>, offset: u8, index_type: &str, code: &str) -> Result<i8, ChessError> {
            let ascii_char = maybe_char.unwrap();
            let ascii_index = ascii_char as u8;
            if ascii_index<offset || ascii_index>=(offset+8) {
                return Err(ChessError{
                    msg: format!("illegal {index_type} char '{ascii_char}' in Position code: {code}"),
                    kind: ErrorKind::IllegalFormat
                })
            };
            Ok((ascii_index - offset) as i8)
        }

        let column = get_index(char_iter.next(), 97, "column", code)?;
        let row = get_index(char_iter.next(), 49, "row", code)?;

        Ok(Position::new_unchecked(column, row))
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.index==other.index
    }
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.index);
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.column_char(), self.row_char())
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct DirectedPosIterator<'a> {
    latest_position: Option<Position>,
    direction: Direction,
    moving_fig_color: Color,
    board: &'a Board,
}

impl DirectedPosIterator<'_> {
    fn new(
        fig_pos: Position,
        fig_color: Color,
        direction: Direction,
        board: &Board,
    ) -> DirectedPosIterator {
        DirectedPosIterator {
            latest_position: Some(fig_pos),
            direction,
            moving_fig_color: fig_color,
            board,
        }
    }
}

impl Iterator for DirectedPosIterator<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let latest_pos = self.latest_position?;
        let new_pos = latest_pos.step(self.direction)?;
        let some_new_pos = Some(new_pos);

        match self.board.get_content_type(new_pos, self.moving_fig_color) {
            FieldContent::OwnFigure => None,
            FieldContent::OpponentFigure => {
                self.latest_position = None;
                some_new_pos
            }
            FieldContent::Empty => {
                self.latest_position = some_new_pos;
                some_new_pos
            }
        }
    }
}

pub struct KnightPosIterator<'a> {
    knight_pos: Position,
    knight_color: Color,
    board: &'a Board,
    index: usize,
}

impl KnightPosIterator<'_> {
    fn new(
        knight_position: Position,
        knight_color: Color,
        board: &Board,
    ) -> KnightPosIterator {
        KnightPosIterator {
            knight_pos: knight_position,
            knight_color,
            board,
            index: 0,
        }
    }
}

impl Iterator for KnightPosIterator<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index==8 {
                break;
            }
            let opt_pos: Option<Position> = match self.index {
                0 =>  self.knight_pos.jump(2, -1),
                1 =>  self.knight_pos.jump(2, 1),
                2 =>  self.knight_pos.jump(-2, -1),
                3 =>  self.knight_pos.jump(-2, 1),
                4 =>  self.knight_pos.jump(1, -2),
                5 =>  self.knight_pos.jump(1, 2),
                6 =>  self.knight_pos.jump(-1, -2),
                7 =>  self.knight_pos.jump(-1, 2),
                _ => panic!("index should lie between [0,7] but is {}", self.index)
            };
            self.index += 1;
            let opt_pos = opt_pos.and_then(|pos|{
                let field_content = self.board.get_content_type(pos, self.knight_color);
                match field_content {
                    FieldContent::OwnFigure => None,
                    _ => Some(pos)
                }
            });
            if opt_pos.is_some() {
                return opt_pos;
            }
        }
        None
    }
}

// Default is needed, so that Move can be stored in a TinyVec/ArrayVec
impl Default for Position {
    fn default() -> Self {
        Position::new_unchecked(0, 0)
    }
}

pub const I8_RANGE_07: Range<i8> = 0..8;


//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use crate::game::game_state::GameState;

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
    column, row, expected_index,
    case(0, 0, 0),
    case(7, 7, 63),
    case(1, 0, 1),
    case(0, 1, 8),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_position_unchecked_new(column: i8, row: i8, expected_index: usize) {
        let pos = Position::new_unchecked(column, row);
        assert_eq!(pos.index, expected_index);
    }

    #[rstest(
    pos, expected_column, expected_row, expected_index,
    case("a1", 0, 0, 0),
    case("h8", 7, 7, 63),
    case("b1", 1, 0, 1),
    case("a2", 0, 1, 8),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_position_from_str(pos: Position, expected_column: i8, expected_row: i8, expected_index: usize) {
        assert_eq!(pos.column, expected_column);
        assert_eq!(pos.row, expected_row);
        assert_eq!(pos.index, expected_index);
    }

    #[rstest(
    pos_str, direction, expected_end_pos_str,
    case("e4", Direction::Up, "e5"),
    case("e4", Direction::UpRight, "f5"),
    case("e4", Direction::Right, "f4"),
    case("e4", Direction::DownRight, "f3"),
    case("e4", Direction::Down, "e3"),
    case("e4", Direction::DownLeft, "d3"),
    case("e4", Direction::Left, "d4"),
    case("e4", Direction::UpLeft, "d5"),
    case("e8", Direction::Up, "none"),
    case("e1", Direction::Down, "none"),
    case("a4", Direction::Left, "none"),
    case("h4", Direction::Right, "none"),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_position_step(pos_str: &str, direction: Direction, expected_end_pos_str: &str) {
        let start_pos = pos_str.parse::<Position>().unwrap();
        let end_pos_string = match start_pos.step(direction) {
            None => {String::from("none")}
            Some(pos) => {format!("{}", pos)}
        } ;
        assert_eq!(end_pos_string, String::from(expected_end_pos_str));
    }

    #[rstest(
    from_str, to_str, expected_direction,
    case("e4", "e6", Some(Direction::Up)),
    case("e4", "g6", Some(Direction::UpRight)),
    case("e4", "g4", Some(Direction::Right)),
    case("e4", "g2", Some(Direction::DownRight)),
    case("e4", "e2", Some(Direction::Down)),
    case("e4", "c2", Some(Direction::DownLeft)),
    case("e4", "c4", Some(Direction::Left)),
    case("e4", "c6", Some(Direction::UpLeft)),
    case("e4", "e4", None),
    case("e4", "a5", None),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_get_direction(from_str: &str, to_str: &str, expected_direction: Option<Direction>) {
        let from = from_str.parse::<Position>().unwrap();
        let to = to_str.parse::<Position>().unwrap();

        let actual_opt_direction = from.get_direction(to);
        assert_eq!(actual_opt_direction, expected_direction);
    }

    #[rstest(
        game_state, knight_pos, expected_count,
        case("white ♔e1 ♚e8", "a1", 2),
        case("white ♔e1 ♚e8", "a8", 2),
        case("black ♔e1 ♚e8", "h1", 2),
        case("black ♔e1 ♚e8", "h8", 2),
        case("black ♔e1 ♚e8", "d4", 8),
        case("white ♔e1 ♘b3 ♚e8", "d4", 7),
        case("white ♔e1 ♘b3 ♚e8 ♞b5", "d4", 7),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_count_reachable_knight_positions(
        game_state: GameState,
        knight_pos: Position,
        expected_count: usize,
    ) {
        let actual_count = knight_pos.count_reachable_knight_positions(game_state.turn_by, &game_state.board);
        assert_eq!(actual_count, expected_count);
    }
}