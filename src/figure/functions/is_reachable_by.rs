use Color::{Black, White};
use Direction::{Down, DownLeft, DownRight, Up, UpLeft, UpRight};
use FigureType::{Bishop, King, Knight, Pawn, Queen, Rook};
use crate::base::color::Color;
use crate::base::direction::{DIAGONAL_DIRECTIONS, Direction, STRAIGHT_DIRECTIONS};
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::figure::figure::{Figure, FigureType};
use crate::game::board::Board;
use crate::game::game_state::GameState;

pub fn get_positions_to_reach_target_from(
    target: Position,
    game_state: &GameState,
) -> Result<Vec<Position>, ChessError> {
    let active_color = game_state.turn_by;

    if let Some(figure) = game_state.board.get_figure(target) {
        // solve castling outside of this method
        if figure.color==active_color {
            return Err(ChessError {
                msg: format!("move captures figure of same color on {target}"),
                kind: ErrorKind::IllegalMove,
            })
        }
    }

    let origins = inner_get_positions_to_reach_target_from(
        target,
        active_color,
        &game_state.board,
        game_state.en_passant_intercept_pos
    );
    Ok(origins)
}

fn inner_get_positions_to_reach_target_from(
    target: Position,
    active_color: Color,
    board: &Board,
    en_passant_intercept_pos: Option<Position>,
) -> Vec<Position> {
    let mut result = Vec::<Position>::with_capacity(4);

    fn find_first_active_figure_on(start: Position, direction: Direction, active_color: Color, board: &Board) -> Option<FoundFigure> {
        let mut current_pos = start;
        let mut distance: usize = 1;
        loop {
            if let Some(pos) = current_pos.step(direction) {
                if let Some(figure) = board.get_figure(pos) {
                    return if figure.color == active_color {
                        Some(FoundFigure {
                            figure_type: figure.fig_type,
                            position: pos,
                            distance,
                        })
                    } else {
                        None
                    };
                };
                distance = distance + 1;
                current_pos = pos;
            } else {
                return None;
            }
        }
    }

    // check bishop, rook, queen, king moves (only normal king moves, no castling)
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on(target, direction, active_color, board) {
                match found_figure.figure_type {
                    Rook | Queen => { result.push(found_figure.position) }
                    King if found_figure.distance == 1 => { result.push(found_figure.position) }
                    _ => {}
                };
            };
        });
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on(target, direction, active_color, board) {
                match found_figure.figure_type {
                    Bishop | Queen => { result.push(found_figure.position) }
                    King if found_figure.distance == 1 => { result.push(found_figure.position) }
                    _ => {}
                };
            };
        });
    }
    // check knight moves
    for pos_from in target.reachable_knight_positions(active_color.toggle(), board) {
        if let Some(Figure{fig_type: Knight, color: knight_color}) = board.get_figure(pos_from){
            if knight_color== active_color {
                result.push(pos_from);
            }
        };
    }
    // check pawn moves
    if (active_color== White && target.row>1) || (active_color== Black && target.row<6) {
        fn contains_active_pawn(pos: Option<Position>, active_color: Color, board: &Board) -> bool {
            pos.map(
                |pos| board.get_figure(pos)
            ).flatten().map(
                |figure| { figure.fig_type == Pawn && figure.color == active_color }
            ).unwrap_or(false)
        }

        let target_pos_is_empty = board.is_empty(target);
        let vertical_direction = if active_color== White { Down} else { Up};
        if target_pos_is_empty {
            // check only straight pawn moves
            let single_step_straight_pos = target.step_unchecked(vertical_direction);
            if contains_active_pawn(Some(single_step_straight_pos), active_color, board) {
                result.push(single_step_straight_pos);
            }

            let target_row_eligible_for_double_step = if active_color== White {3} else {4};
            if target.row== target_row_eligible_for_double_step && board.is_empty(single_step_straight_pos) {
                // check double step pawn move
                let double_step_straight_pos = single_step_straight_pos.step_unchecked(vertical_direction);
                if contains_active_pawn(Some(double_step_straight_pos), active_color, board) {
                    result.push(single_step_straight_pos);
                }
            }
        }
        if !target_pos_is_empty || en_passant_intercept_pos.map(|intercept_pos|target==intercept_pos).unwrap_or(false) {
            // check only diagonal moves

            let attack_pawn_directions: [Direction; 2] = if active_color== White {
                [DownLeft, DownRight]
            } else {
                [UpLeft, UpRight]
            };
            attack_pawn_directions.map(|direction: Direction|target.step(direction)).iter().for_each(|&opt_pos|{
                if let Some(pos) = opt_pos {
                    if let Some(figure)= board.get_figure(pos) {
                        if figure.fig_type == Pawn && figure.color==active_color {
                            result.push(pos);
                        }
                    };
                }
            });
        }
    }

    result
}

struct FoundFigure {
    figure_type: FigureType,
    position: Position,
    distance: usize,
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::base::util::tests::vec_to_str;
    use super::*;

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        game_state, target, expected_nr_of_origins,
        case("", "b3", 1),
        case("", "b4", 1),
        case("", "b5", 0),
        case("h2h3", "b6", 1),
        case("h2h3", "b5", 1),
        case("h2h3", "b4", 0),
        case("", "c3", 2),
        case("a2a3", "f6", 2),
        case("b1c3 g8f6", "d5", 1),
        case("b1c3 g8f6", "e4", 2),
        case("e2e4 e7e5", "e2", 4),
        case("e2e4 e7e5", "e3", 0),
        case("e2e3 d7d5 e3e4", "d7", 4),
        case("a2a4 b7b5", "b5", 1),
        case("a2a4 h7h5", "a5", 1),
        case("a2a4 h7h5 g2g4", "g4", 1),
        case("a2a4 h7h5 a4a5 h5h4 g2g4", "g3", 1),
        case("a2a4 h7h5 h2h4", "h4", 0),
        case("a2a4 h7h5 a4a5 b7b5", "b6", 1),
        case("a2a4 b7b5 a4a5 h7h5", "b6", 0),
        case("white ♕c2 ♘b3 ♘b5 ♘c6 ♘e6 ♞f5 ♘f3 ♘e2 ♔h1 ♚e8", "d4", 6),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_get_positions_to_reach_target_from(
        game_state: GameState,
        target: Position,
        expected_nr_of_origins: usize,
    ) {
        let origins = get_positions_to_reach_target_from(target, &game_state).unwrap();
        let actual_nr_of_origins = origins.len();
        assert_eq!(
            actual_nr_of_origins, expected_nr_of_origins,
            "computed origins for {target}: {}", vec_to_str(&origins, ", ")
        );
    }
}