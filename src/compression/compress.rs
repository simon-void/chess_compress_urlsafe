use crate::base::a_move::Move;
use crate::compression::base64::encode_base64;
use crate::figure::functions::is_reachable_by::get_positions_to_reach_target_from;
use crate::base::color::Color;
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::game::game_state::GameState;

pub fn compress(moves: Vec<Move>) -> Result<String, ChessError> {
    let mut game_state = GameState::classic();
    let mut encoded_moves = String::with_capacity(moves.len()*2);


    println!("start compressing {} moves", moves.len());

    let mut half_move_index = 0;
    for next_move in moves.into_iter() {
        let active_color = game_state.turn_by;
        let target_pos = next_move.from_to.to;
        let from_pos_can_be_dropped = {
            if game_state.looks_like_castling(next_move.from_to)? {
                false
            } else {
                let positions_with_figures_that_can_reach_target: Vec<Position> = get_positions_to_reach_target_from(target_pos, &game_state)?;
                if !positions_with_figures_that_can_reach_target.contains(&next_move.from_to.from) {
                    let move_nr = 1 + half_move_index / 2;
                    let err_msg = match active_color {
                        Color::White => format!("move {move_nr}. {next_move} .. is illegal since you can't go there from {}", next_move.from_to.from),
                        Color::Black => format!("move {move_nr}. .. {next_move} is illegal since you can't go there from {}", next_move.from_to.from),
                    };
                    return Err(ChessError {
                        msg: err_msg,
                        kind: ErrorKind::IllegalMove,
                    });
                };
                positions_with_figures_that_can_reach_target.len() == 1
            }
        };

        if from_pos_can_be_dropped {
            // only to-position is required to reconstruct whole FromTo
            encoded_moves.push(encode_base64(next_move.from_to.to));
        } else {
            // encode from- and to-positions
            encoded_moves.push(encode_base64(next_move.from_to.from));
            encoded_moves.push(encode_base64(next_move.from_to.to));
        };
        if let Some(promotion_type) = next_move.promotion_type {
            encoded_moves.push(promotion_type.as_encoded());
        };

        game_state = game_state.do_move(next_move).0;
        half_move_index = half_move_index + 1;
    }

    Ok(encoded_moves)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest(
        joined_moves, expected_encoded_game_with_moves_separated_by_space,
        case("", ""), // | "no moves -> empty encoded String
        case("c2c3", "KS"), // KS | destination not unique target -> encoding needs two chars
        case("c2c4", "a"), // Ka | destination is unique target -> encoding needs one char
        case("a2a4, h7h6, a4a5, b7b5, a5b6, h6h5, b6c7, h5h4, g2g3, h4g3, c7d8Q", "Y 3v g h p n y f W W 7Q"), // IY 3v Yg xh gp vn py nf OW fW y7Q | tests all pawn moves single-step, double-step, diagonal-capture, en-passant & promotion
        case("d2d3, g7g6, c1e3, f8g7, b1c3, g8f6, d1d2, e8h8, e1a1", "T u CU 2 BS -t DL _ A") // LT 2u CU 92 BS -t DL 8_ EA | tests king- & queen-side castling
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_compress(
        joined_moves: &str,
        expected_encoded_game_with_moves_separated_by_space: &str,
    ) {
        let moves: Vec<Move> = joined_moves.split(',').filter(|it| it.len() > 0).map(|it| it.trim().parse().unwrap()).collect();
        let expected_encoded_game = expected_encoded_game_with_moves_separated_by_space.replace(' ', "");
        let actual_encoded_game = compress(moves).unwrap();
        assert_eq!(actual_encoded_game, expected_encoded_game);
    }
}
