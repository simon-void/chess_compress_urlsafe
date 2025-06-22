use FigureType::Knight;
use crate::a_move::FromTo;
use crate::base::color::Color;
use crate::base::direction::{DIAGONAL_DIRECTIONS, STRAIGHT_DIRECTIONS};
use crate::base::position::Position;
use crate::figure::functions::is_reachable_by::find_first_active_figure_on_line_from;
use crate::FigureType;
use crate::FigureType::{Bishop, King, Queen, Rook, Pawn};
use crate::game::board::Board;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OriginStatus {
    Unambiguous, ColumnIsAmbiguous, RowIsAmbiguous, ColumnAndRowAreAmbiguous 
}

pub fn is_origin_of_move_ambiguous(board_before_move: &Board, from_to: FromTo) -> OriginStatus {
    let moving_figure = if let Some(figure) = board_before_move.get_figure(from_to.from) {
        figure
    } else {
        return OriginStatus::ColumnAndRowAreAmbiguous;
    };

    match moving_figure.fig_type {
        King => { OriginStatus::Unambiguous }
        Pawn => {
            if from_to.from.column != from_to.to.column {
                let other_pawn_column = from_to.from.column + 2 * (from_to.to.column - from_to.from.column);
                if (0..8).contains(&other_pawn_column) {
                    let other_pawn_pos = Position::new_unchecked(other_pawn_column, from_to.from.row);
                    if board_before_move.contains_figure(other_pawn_pos, Pawn, moving_figure.color) {
                        return OriginStatus::ColumnIsAmbiguous;
                    }
                }
            }
            OriginStatus::Unambiguous
        }
        Rook => { is_origin_of_rook_move_ambiguous(board_before_move, from_to, moving_figure.color) }
        Knight => { is_origin_of_knight_move_ambiguous(board_before_move, from_to, moving_figure.color) }
        Bishop => { is_origin_of_bishop_move_ambiguous(board_before_move, from_to, moving_figure.color) }
        Queen => { is_origin_of_queen_move_ambiguous(board_before_move, from_to, moving_figure.color) }
    }
}

fn is_origin_of_bishop_move_ambiguous(board: &Board, from_to: FromTo, bishop_color: Color) -> OriginStatus {
    let mut alt_bishop_pos = Vec::<Position>::with_capacity(1);
    {
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(from_to.to, direction, bishop_color, board) {
                if found_figure.position!=from_to.from && found_figure.figure_type==Bishop {
                    alt_bishop_pos.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(from_to.from, &alt_bishop_pos)
}

fn is_origin_of_rook_move_ambiguous(board: &Board, from_to: FromTo, rook_color: Color) -> OriginStatus {
    let mut alt_rook_pos = Vec::<Position>::with_capacity(2);
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(from_to.to, direction, rook_color, board) {
                if found_figure.position!=from_to.from && found_figure.figure_type==Rook {
                    alt_rook_pos.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(from_to.from, &alt_rook_pos)
}

fn is_origin_of_queen_move_ambiguous(board: &Board, from_to: FromTo, queen_color: Color) -> OriginStatus {
    let mut alt_queen_pos = Vec::<Position>::with_capacity(2);
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(from_to.to, direction, queen_color, board) {
                if found_figure.position!=from_to.from && found_figure.figure_type==Queen {
                    alt_queen_pos.push(found_figure.position)
                }
            };
        });
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on_line_from(from_to.to, direction, queen_color, board) {
                if found_figure.position!=from_to.from && found_figure.figure_type==Queen {
                    alt_queen_pos.push(found_figure.position)
                }
            };
        });
    }
    get_ambiguous_status(from_to.from, &alt_queen_pos)
}

fn is_origin_of_knight_move_ambiguous(board: &Board, from_to: FromTo, knight_color: Color) -> OriginStatus {
    let mut alt_knight_pos = Vec::<Position>::with_capacity(4);
    {
        from_to.to.reachable_knight_positions(knight_color.toggle(), board).for_each(|pos| {
            if pos!=from_to.from && board.contains_figure(pos, Knight, knight_color) {
                alt_knight_pos.push(pos)
            };
        });
    }
    get_ambiguous_status(from_to.from, &alt_knight_pos)
}

#[allow(clippy::collapsible_else_if)]
fn get_ambiguous_status(pos: Position, vec_of_alt_pos: &[Position]) -> OriginStatus {
    let alt_pos_with_same_column_found = vec_of_alt_pos.iter().any(|&alt_pos|{alt_pos.column==pos.column});
    let alt_pos_with_same_row_found = vec_of_alt_pos.iter().any(|&alt_pos|{alt_pos.row==pos.row});
    if alt_pos_with_same_column_found {
        if alt_pos_with_same_row_found {
            OriginStatus::ColumnAndRowAreAmbiguous
        } else {
            OriginStatus::RowIsAmbiguous
        }
    } else {
        if alt_pos_with_same_row_found {
            OriginStatus::ColumnIsAmbiguous
        } else {
            OriginStatus::Unambiguous
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
        case("white ♔e1 ♖b2 ♖c7 ♚e8", "b2b7", Unambiguous),
        case("white ♔e1 ♖b2 ♖b7 ♖c3 ♚e8", "b2b3", RowIsAmbiguous),
        case("white ♔e1 ♖b2 ♜b7 ♜c3 ♚e8", "b2b3", Unambiguous),
        case("white ♔e1 ♘b2 ♘c7 ♚e8", "b2c4", Unambiguous),
        case("white ♔e1 ♘b2 ♘b6 ♚e8", "b2c4", RowIsAmbiguous),
        case("white ♔e1 ♘b2 ♘d2 ♚e8", "b2c4", ColumnIsAmbiguous),
        case("white ♔e1 ♘b2 ♘d2 ♘b6 ♚e8", "b2c4", ColumnAndRowAreAmbiguous),
        case("white ♔e1 ♘b2 ♞d2 ♞b6 ♚e8", "b2c4", Unambiguous),
        case("white ♔e1 ♗c2 ♝c6 ♝g2 ♚e8", "c2e4", Unambiguous),
        case("white ♔e1 ♗c2 ♗c6 ♝g2 ♚e8", "c2e4", RowIsAmbiguous),
        case("white ♔e1 ♗c2 ♝c6 ♗g2 ♚e8", "c2e4", ColumnIsAmbiguous),
        case("white ♔e1 ♗c2 ♗c6 ♗g2 ♚e8", "c2e4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕b2 ♕d2 ♕f2 ♕b4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "b2d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♕a1 ♕d2 ♕f2 ♕b4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a1d4", Unambiguous),
        case("white ♔h1 ♕b2 ♕d2 ♕f2 ♕a4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", ColumnIsAmbiguous),
        case("white ♔h1 ♕a1 ♕d2 ♕f2 ♕a4 ♕f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", ColumnAndRowAreAmbiguous),
        case("white ♔h1 ♗a1 ♕d2 ♕f2 ♕a4 ♖f4 ♕b6 ♕d6 ♕f6 ♚h7", "a4d4", Unambiguous),
        case("white ♔e1 ♘b2 ♖b4 ♗b5 ♕c2 ♚e8", "b2c4", Unambiguous),
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
