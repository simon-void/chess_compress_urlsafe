use crate::base::Color;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Up, UpRight, Right, DownRight, Down, DownLeft, Left, UpLeft
}

impl Direction {
    /**
    * returns a triple: (forwardLeft, forward, forwardRight)
    */
    pub fn forward_directions(color: Color) -> (Direction, Direction, Direction) {
        match color {
            Color::White => (Direction::UpLeft, Direction::Up,Direction::UpRight),
            Color::Black => (Direction::DownLeft, Direction::Down,Direction::DownRight),
        }
    }

    pub fn reverse(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::UpRight => Direction::DownLeft,
            Direction::Right => Direction::Left,
            Direction::DownRight => Direction::UpLeft,
            Direction::Down => Direction::Up,
            Direction::DownLeft => Direction::UpRight,
            Direction::Left => Direction::Right,
            Direction::UpLeft => Direction::DownRight,
        }
    }

    pub fn is_straight(&self) -> bool {
        matches!(self, Direction::Up | Direction::Down | Direction::Left | Direction::Right)
    }

    pub fn is_diagonal(&self) -> bool {
        matches!(self, Direction::UpLeft | Direction::UpRight | Direction::DownLeft | Direction::DownRight)
    }
}

pub static ALL_DIRECTIONS: [Direction; 8] = [
    Direction::Up, Direction::UpRight, Direction::Right, Direction::DownRight,
    Direction::Down, Direction::DownLeft, Direction::Left, Direction::UpLeft
];

pub static STRAIGHT_DIRECTIONS: [Direction; 4] = [
    Direction::Up, Direction::Right, Direction::Down, Direction::Left
];

pub static DIAGONAL_DIRECTIONS: [Direction; 4] = [
    Direction::UpRight, Direction::DownRight, Direction::DownLeft, Direction::UpLeft
];
