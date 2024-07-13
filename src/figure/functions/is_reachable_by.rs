use crate::base::color::Color;
use crate::base::direction::{DIAGONAL_DIRECTIONS, Direction, STRAIGHT_DIRECTIONS};
use crate::base::position::Position;
use crate::figure::figure::{Figure, FigureType};
use crate::game::board::Board;
use crate::game::game_state::GameState;

pub fn get_positions_to_reach_target_from(
    target: Position,
    game_state: &GameState,
) -> Vec<Position> {
    inner_get_positions_to_reach_target_from(target, game_state.turn_by, &game_state.board, game_state.en_passant_intercept_pos)
}

fn inner_get_positions_to_reach_target_from(
    target: Position,
    active_color: Color,
    board: &Board,
    en_passant_intercept_pos: Option<Position>,
) -> Vec<Position> {
    let mut result = Vec::<Position>::with_capacity(4);

    fn find_first_active_figure_on(start: Position, direction: Direction, active_color: Color, board: &Board) -> Option<FoundFigure> {
        let pos = start;
        let mut distance: usize = 1;
        loop {
            if let Some(pos) = pos.step(direction) {
                if let Some(figure) = board.get_figure(pos) {
                    if figure.color==active_color {
                        return Some(FoundFigure{
                            figure_type: figure.fig_type,
                            position: pos,
                            distance,
                        })
                    } else {
                        return None
                    }
                }
                distance = distance + 1;
            } else {
                return None
            }
        }
    }

    // check bishop, rook, queen, king moves (only normal king moves, no castling)
    {
        STRAIGHT_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on(target, direction, active_color, board) {
                match found_figure.figure_type {
                    FigureType::Rook | FigureType::Queen => { result.push(found_figure.position) }
                    FigureType::King if found_figure.distance == 1 => { result.push(found_figure.position) }
                    _ => {}
                };
            };
        });
        DIAGONAL_DIRECTIONS.iter().for_each(|&direction| {
            if let Some(found_figure) = find_first_active_figure_on(target, direction, active_color, board) {
                match found_figure.figure_type {
                    FigureType::Bishop | FigureType::Queen => { result.push(found_figure.position) }
                    FigureType::King if found_figure.distance == 1 => { result.push(found_figure.position) }
                    _ => {}
                };
            };
        });
    }
    // check knight moves
    for pos_from in target.reachable_knight_positions(active_color.toggle(), board) {
        if let Some(figure) = board.get_figure(pos_from) {
            if figure.fig_type==FigureType::Knight {
                result.push(pos_from)
            }
        }
        match board.get_figure(pos_from) {
            Some(Figure{fig_type: FigureType::Knight, color: _}) => {result.push(pos_from)}
            _ => {}
        };
    }
    // check pawn moves
    if (active_color==Color::White && target.row>1) || (active_color==Color::Black && target.row<6) {
        fn contains_active_pawn(pos: Option<Position>, active_color: Color, board: &Board) -> bool {
            pos.map(
                |pos| board.get_figure(pos)
            ).flatten().map(
                |figure| { figure.fig_type == FigureType::Pawn && figure.color == active_color }
            ).unwrap_or(false)
        }

        let target_pos_is_empty = board.is_empty(target);
        let vertical_direction = if active_color==Color::White {Direction::Down} else {Direction::Up};
        if target_pos_is_empty {
            // check only straight pawn moves
            let single_step_straight_pos = target.step_unchecked(vertical_direction);
            if contains_active_pawn(Some(single_step_straight_pos), active_color, board) {
                result.push(single_step_straight_pos);
            }

            let target_row_eligible_for_double_step = if active_color==Color::White {3} else {4};
            if target.column== target_row_eligible_for_double_step && board.is_empty(single_step_straight_pos) {
                // check double step pawn move
                let double_step_straight_pos = single_step_straight_pos.step_unchecked(vertical_direction);
                if contains_active_pawn(Some(double_step_straight_pos), active_color, board) {
                    result.push(single_step_straight_pos);
                }
            }
        }
        if !target_pos_is_empty || en_passant_intercept_pos.map(|intercept_pos|target==intercept_pos).unwrap_or(false) {
            // check only diagonal moves

            let attack_pawn_directions: [Direction; 2] = if active_color==Color::White {
                [Direction::DownLeft, Direction::DownRight]
            } else {
                [Direction::UpLeft, Direction::UpRight]
            };
            attack_pawn_directions.map(|direction: Direction|target.step(direction)).iter().for_each(|&opt_pos|{
                if let Some(pos) = opt_pos {
                    if let Some(figure)= board.get_figure(pos) {
                        if figure.fig_type == FigureType::Pawn && figure.color==active_color {
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
    use super::*;

    #[test]
    fn testing_for_get_positions_to_reach_target_from() {
        panic!("TODO")
        // let mut move_collection: Moves = tiny_vec!();
        // for_reachable_knight_moves(
        //     Color::White,
        //     "b1".parse::<Position>().unwrap(),
        //     &Board::classic(),
        //     &mut move_collection,
        // );
        // assert_eq!(move_collection.len(), 2);
    }
}