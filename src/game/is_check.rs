use FigureType::King;
use crate::base::color::Color::White;
use crate::base::direction::{Direction, DIAGONAL_DIRECTIONS, STRAIGHT_DIRECTIONS};
use crate::base::direction::Direction::{DownLeft, DownRight, UpLeft, UpRight};
use crate::base::position::Position;
use crate::figure::functions::is_reachable_by::find_first_active_figure_on_line_from;
use crate::FigureType;
use crate::FigureType::{Bishop, Knight, Pawn, Queen, Rook};
use crate::game::board::Board;

pub fn is_check(board: &Board, king_pos: Position) -> bool {
    let attacker_color = {
        let figure = board.get_figure(king_pos).unwrap_or_else(|| panic!("no figure on board on position {}! board: {}", king_pos, board));
        if figure.fig_type != King {
            panic!("given position {} leads to a {}, not a King on board: {}", king_pos, figure.fig_type, board);
        } 
        figure.color.toggle()
    };

    for direction in STRAIGHT_DIRECTIONS {
        if let Some(attacker) = find_first_active_figure_on_line_from(king_pos, direction, attacker_color, board) {
            if attacker.figure_type == Queen || attacker.figure_type == Rook {
                return true;
            }
        }
    }

    for direction in DIAGONAL_DIRECTIONS {
        if let Some(attacker) = find_first_active_figure_on_line_from(king_pos, direction, attacker_color, board) {
            if attacker.figure_type == Queen || attacker.figure_type == Bishop {
                return true;
            }
        }
    }


    for knight_jump_pos in king_pos.reachable_knight_positions(attacker_color.toggle(), board) {
        if let Some(attacker) = board.get_figure(knight_jump_pos) {
            if attacker.fig_type == Knight {
                return true;
            }
        };
    }

    // look for check by pawn
    let attack_pawn_directions: [Direction; 2] = if attacker_color == White {
        [DownLeft, DownRight]
    } else {
        [UpLeft, UpRight]
    };
    for pawn_pos in attack_pawn_directions.map(|direction: Direction|king_pos.step(direction)).into_iter().flatten() {
        if let Some(figure) = board.get_figure(pawn_pos) {
            if figure.fig_type == Pawn && figure.color == attacker_color {
                return true;
            }
        }
    }

    false
}
