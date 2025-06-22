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


//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::base::position::Position;
    use crate::game::game_state::GameState;
    use crate::game::is_check::is_check;
    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        game_state, king_pos, expected_is_check,
        case("white ♔e3 ♟f4 ♚e8", "e3", true),
        case("white ♔e3 ♟d4 ♚e8", "e3", true),
        case("white ♔e3 ♟e4 ♚e8", "e3", false),
        case("white ♔e3 ♟d3 ♚e8", "e3", false),
        case("white ♔e3 ♟f3 ♚e8", "e3", false),
        case("white ♔e3 ♟f2 ♚e8", "e3", false),
        case("white ♔e3 ♟e2 ♚e8", "e3", false),
        case("white ♔e3 ♟d2 ♚e8", "e3", false),
        case("white ♔e3 ♟g5 ♚e8", "e3", false),
        case("white ♔e3 ♟c5 ♚e8", "e3", false),

        case("white ♔e3 ♜c5 ♚e8", "e3", false),
        case("white ♔e3 ♜d5 ♚e8", "e3", false),
        case("white ♔e3 ♜e5 ♚e8", "e3", true),
        case("white ♔e3 ♜f5 ♚e8", "e3", false),
        case("white ♔e3 ♜g5 ♚e8", "e3", false),
        case("white ♔e3 ♜c4 ♚e8", "e3", false),
        case("white ♔e3 ♜c3 ♚e8", "e3", true),
        case("white ♔e3 ♜c2 ♚e8", "e3", false),
        case("white ♔e3 ♜g4 ♚e8", "e3", false),
        case("white ♔e3 ♜g3 ♚e8", "e3", true),
        case("white ♔e3 ♜g2 ♚e8", "e3", false),
        case("white ♔e3 ♜c1 ♚e8", "e3", false),
        case("white ♔e3 ♜d1 ♚e8", "e3", false),
        case("white ♔e3 ♜e1 ♚e8", "e3", true),
        case("white ♔e3 ♜f1 ♚e8", "e3", false),
        case("white ♔e3 ♜g1 ♚e8", "e3", false),

        case("white ♔e3 ♞c5 ♚e8", "e3", false),
        case("white ♔e3 ♞d5 ♚e8", "e3", true),
        case("white ♔e3 ♞e5 ♚e8", "e3", false),
        case("white ♔e3 ♞f5 ♚e8", "e3", true),
        case("white ♔e3 ♞g5 ♚e8", "e3", false),
        case("white ♔e3 ♞c4 ♚e8", "e3", true),
        case("white ♔e3 ♞c3 ♚e8", "e3", false),
        case("white ♔e3 ♞c2 ♚e8", "e3", true),
        case("white ♔e3 ♞g4 ♚e8", "e3", true),
        case("white ♔e3 ♞g3 ♚e8", "e3", false),
        case("white ♔e3 ♞g2 ♚e8", "e3", true),
        case("white ♔e3 ♞c1 ♚e8", "e3", false),
        case("white ♔e3 ♞d1 ♚e8", "e3", true),
        case("white ♔e3 ♞e1 ♚e8", "e3", false),
        case("white ♔e3 ♞f1 ♚e8", "e3", true),
        case("white ♔e3 ♞g1 ♚e8", "e3", false),

        case("white ♔e3 ♝c5 ♚e8", "e3", true),
        case("white ♔e3 ♝d5 ♚e8", "e3", false),
        case("white ♔e3 ♝e5 ♚e8", "e3", false),
        case("white ♔e3 ♝f5 ♚e8", "e3", false),
        case("white ♔e3 ♝g5 ♚e8", "e3", true),
        case("white ♔e3 ♝c4 ♚e8", "e3", false),
        case("white ♔e3 ♝c3 ♚e8", "e3", false),
        case("white ♔e3 ♝c2 ♚e8", "e3", false),
        case("white ♔e3 ♝g4 ♚e8", "e3", false),
        case("white ♔e3 ♝g3 ♚e8", "e3", false),
        case("white ♔e3 ♝g2 ♚e8", "e3", false),
        case("white ♔e3 ♝c1 ♚e8", "e3", true),
        case("white ♔e3 ♝d1 ♚e8", "e3", false),
        case("white ♔e3 ♝e1 ♚e8", "e3", false),
        case("white ♔e3 ♝f1 ♚e8", "e3", false),
        case("white ♔e3 ♝g1 ♚e8", "e3", true),

        case("white ♔e3 ♛c5 ♚e8", "e3", true),
        case("white ♔e3 ♛d5 ♚e8", "e3", false),
        case("white ♔e3 ♛e5 ♚e8", "e3", true),
        case("white ♔e3 ♛f5 ♚e8", "e3", false),
        case("white ♔e3 ♛g5 ♚e8", "e3", true),
        case("white ♔e3 ♛c4 ♚e8", "e3", false),
        case("white ♔e3 ♛c3 ♚e8", "e3", true),
        case("white ♔e3 ♛c2 ♚e8", "e3", false),
        case("white ♔e3 ♛g4 ♚e8", "e3", false),
        case("white ♔e3 ♛g3 ♚e8", "e3", true),
        case("white ♔e3 ♛g2 ♚e8", "e3", false),
        case("white ♔e3 ♛c1 ♚e8", "e3", true),
        case("white ♔e3 ♛d1 ♚e8", "e3", false),
        case("white ♔e3 ♛e1 ♚e8", "e3", true),
        case("white ♔e3 ♛f1 ♚e8", "e3", false),
        case("white ♔e3 ♛g1 ♚e8", "e3", true),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_is_check(
        game_state: GameState,
        king_pos: Position,
        expected_is_check: bool,
    ) {
        // game_state.
        let actual_is_check = is_check(&game_state.board, king_pos);
        assert_eq!(actual_is_check, expected_is_check);
    }
}
