use std::collections::HashSet;
use FigureType::Knight;
use OriginStatus::{ColumnAndRowAreAmbiguous, ColumnIsAmbiguous, RowIsAmbiguous};
use crate::a_move::FromTo;
use crate::base::color::Color;
use crate::base::direction::{DIAGONAL_DIRECTIONS, STRAIGHT_DIRECTIONS};
use crate::base::position::Position;
use crate::figure::functions::is_reachable_by::find_first_active_figure_on_line_from;
use crate::FigureType;
use crate::FigureType::{Bishop, King, Queen, Rook, Pawn};
use crate::game::board::Board;
use crate::OriginStatus::Unambiguous;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OriginStatus {
    Unambiguous, ColumnIsAmbiguous, RowIsAmbiguous, ColumnAndRowAreAmbiguous 
}

pub fn is_origin_of_move_ambiguous(board_before_move: &Board, from_to: FromTo) -> OriginStatus {
    let moving_figure = if let Some(figure) = board_before_move.get_figure(from_to.from) {
        figure
    } else {
        return ColumnAndRowAreAmbiguous;
    };

    match moving_figure.fig_type {
        King => { Unambiguous }
        Pawn => {
            if from_to.from.column != from_to.to.column {
                let other_pawn_column = from_to.from.column + 2 * (from_to.to.column - from_to.from.column);
                if (0..8).contains(&other_pawn_column) {
                    let other_pawn_pos = Position::new_unchecked(other_pawn_column, from_to.from.row);
                    if board_before_move.contains_figure(other_pawn_pos, Pawn, moving_figure.color) {
                        return ColumnIsAmbiguous;
                    }
                }
            }
            Unambiguous
        }
        Rook => { is_origin_of_rook_move_ambiguous(board_before_move, from_to.to, moving_figure.color) }
        Knight => { is_origin_of_knight_move_ambiguous(board_before_move, from_to.to, moving_figure.color) }
        Bishop => { is_origin_of_bishop_move_ambiguous(board_before_move, from_to.to, moving_figure.color) }
        Queen => { is_origin_of_queen_move_ambiguous(board_before_move, from_to.to, moving_figure.color) }
    }
}

fn is_origin_of_bishop_move_ambiguous(board: &Board, attacked_pos: Position, bishop_color: Color) -> OriginStatus {
    let mut bishop_positions = Vec::<Position>::with_capacity(1);
    {
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(attacked_pos, direction, bishop_color, board) {
                if found_figure.figure_type==Bishop {
                    bishop_positions.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(&bishop_positions)
}

fn is_origin_of_rook_move_ambiguous(board: &Board, attacked_pos: Position, rook_color: Color) -> OriginStatus {
    let mut rook_positions = Vec::<Position>::with_capacity(2);
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(attacked_pos, direction, rook_color, board) {
                if found_figure.figure_type==Rook {
                    rook_positions.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(&rook_positions)
}

fn is_origin_of_queen_move_ambiguous(board: &Board, attacked_pos: Position, queen_color: Color) -> OriginStatus {
    let mut queen_positions = Vec::<Position>::with_capacity(2);
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(attacked_pos, direction, queen_color, board) {
                if found_figure.figure_type==Queen {
                    queen_positions.push(found_figure.position)
                }
            };
        });
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(attacked_pos, direction, queen_color, board) {
                if found_figure.figure_type==Queen {
                    queen_positions.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(&queen_positions)
}

fn is_origin_of_knight_move_ambiguous(board: &Board, attacked_pos: Position, knight_color: Color) -> OriginStatus {
    let mut knight_positions = Vec::<Position>::with_capacity(4);
    {
        attacked_pos.reachable_knight_positions(knight_color.toggle(), board).for_each(|pos| {
            if board.contains_figure(pos, Knight, knight_color) {
                knight_positions.push(pos)
            };
        });
    }
    get_ambiguous_status(&knight_positions)
}

#[allow(clippy::collapsible_else_if)]
fn get_ambiguous_status(vec_of_pos: &[Position]) -> OriginStatus {
    let number_of_attackers = vec_of_pos.len();
    if number_of_attackers == 1 {
        return Unambiguous;
    }
    
    let mut columns = HashSet::with_capacity(number_of_attackers);
    let mut rows = HashSet::with_capacity(number_of_attackers);
    vec_of_pos.iter().for_each(|alt_pos| {
        columns.insert(alt_pos.column);
        rows.insert(alt_pos.row);
    });
    
    let is_column_ambiguous = columns.len()>1;
    let is_row_ambiguous = rows.len()>1;
    
    if is_column_ambiguous {
        if is_row_ambiguous {
            ColumnAndRowAreAmbiguous
        } else {
            ColumnIsAmbiguous
        }
    } else {
        if is_row_ambiguous {
            RowIsAmbiguous
        } else {
            Unambiguous
        }
    }
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::a_move::FromTo;
    use crate::base::ambiguous_origin::{is_origin_of_move_ambiguous, OriginStatus};
    use crate::base::ambiguous_origin::OriginStatus::*;
    use crate::game::game_state::GameState;
    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        game_state, from_to, expected_status,
        case("white ♔e1 ♚e8", "d4d5", ColumnAndRowAreAmbiguous), // FromTo.from is empty
        case("white ♔e1 ♚e8", "e1e2", Unambiguous),
        case("white ♔e1 ♙a2 ♚e8", "a2a3", Unambiguous),
        case("white ♔e1 ♙a2 ♚e8", "a2a4", Unambiguous),
        case("white ♔e1 ♙a2 ♟b3 ♚e8", "a2b3", Unambiguous),
        case("white ♔e1 ♙b2 ♟a3 ♚e8", "b2a3", Unambiguous),
        case("white ♔e1 ♙a2 ♙c2 ♟b3 ♚e8", "a2a3", Unambiguous),
        case("white ♔e1 ♙a2 ♙c2 ♟b3 ♚e8", "a2b3", ColumnIsAmbiguous),
        case("white ♔e1 ♙a2 ♟c2 ♟b3 ♚e8", "a2b3", Unambiguous),
        case("white ♔e1 ♖b2 ♖g2 ♚e8", "b2a2", Unambiguous),
        case("white ♔e1 ♖b2 ♖g2 ♚e8", "b2b3", Unambiguous),
        case("white ♔e1 ♖b2 ♖g2 ♚e8", "b2e2", ColumnIsAmbiguous),
        case("white ♔e1 ♖b2 ♜d2 ♖g2 ♚e8", "b2d2", ColumnIsAmbiguous),
        case("white ♔e1 ♖b2 ♜d2 ♖g2 ♚e8", "b2c2", Unambiguous),
        case("white ♔e2 ♖b2 ♖g2 ♚e8", "b2d2", Unambiguous),
        case("white ♔e1 ♖b2 ♖b7 ♚e8", "b2b3", RowIsAmbiguous),
        case("white ♔e1 ♖b2 ♖c7 ♚e8", "b2b7", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♖b2 ♖b7 ♖c3 ♚e8", "b2b3", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♖b2 ♜b7 ♜c3 ♚e8", "b2b3", Unambiguous),
        case("white ♔e1 ♘b2 ♘c7 ♚e8", "b2c4", Unambiguous),
        case("white ♔e1 ♘b2 ♘b6 ♚e8", "b2c4", RowIsAmbiguous),
        case("white ♔e1 ♘b2 ♘d2 ♚e8", "b2c4", ColumnIsAmbiguous),
        case("white ♔e1 ♘b2 ♘d2 ♘b6 ♚e8", "b2c4", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♘b2 ♘d6 ♚e8", "b2c4", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♘b2 ♞d2 ♞b6 ♚e8", "b2c4", Unambiguous),
        case("white ♔e1 ♗c2 ♝c6 ♝g2 ♚e8", "c2e4", Unambiguous),
        case("white ♔e1 ♗c2 ♗c6 ♝g2 ♚e8", "c2e4", RowIsAmbiguous),
        case("white ♔e1 ♗c2 ♝c6 ♗g2 ♚e8", "c2e4", ColumnIsAmbiguous),
        case("white ♔e1 ♗c2 ♗c6 ♗g2 ♚e8", "c2e4", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♗a1 ♗h8 ♚e8", "a1d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕b2 ♕d2 ♕f2 ♕b4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "b2d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕a1 ♕d2 ♕f2 ♕b4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a1d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕b2 ♕d2 ♕f2 ♕a4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕a1 ♕d2 ♕f2 ♕a4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♗a1 ♕d2 ♕f2 ♕a4 ♖f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♘b2 ♖b4 ♗b5 ♕c2 ♚e8", "b2c4", Unambiguous),
        case("white ♔h4 ♙g2 ♙h3 ♕e8 ♕g3 ♚h1", "e8e1", ColumnAndRowAreAmbiguous),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_ambiguous_origin_status(
        game_state: GameState,
        from_to: FromTo,
        expected_status: OriginStatus,
    ) {
        let actual_status = is_origin_of_move_ambiguous(&game_state.board, from_to);
        assert_eq!(actual_status, expected_status);
    }
}
