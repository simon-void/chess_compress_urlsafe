use std::str::Chars;
use crate::base::a_move::{FromTo, Move, MoveData, PromotionType};
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::compression::base64::{assert_is_url_safe_base64, decode_base64};
use crate::figure::functions::is_reachable_by::get_positions_to_reach_target_from;
use crate::game::game_state::GameState;

pub fn decompress(base64_encoded_match: &str) -> Result<Vec<MoveData>, ChessError> {
    assert_is_url_safe_base64(base64_encoded_match)?;

    fn get_next_position(encoded_chars: &mut Chars) -> Result<Option<Position>, ChessError> {
        match encoded_chars.next() {
            None => { Ok(None) }
            Some(base64_char) => {
                let position = decode_base64(base64_char)?;
                Ok(Some(position))
            }
        }
    }

    let mut encoded_chars: Chars = base64_encoded_match.chars();
    let mut game_state = GameState::classic();
    let mut moves_played: Vec<MoveData> = Vec::new();

    let mut half_move_index = 0;
    loop {
        let move_index = half_move_index / 2;

        let next_move = {
            let active_color = game_state.turn_by;
            let first_pos: Position = match get_next_position(&mut encoded_chars)? {
                None => { break; }
                Some(pos) => { pos }
            };

            let from_to = if game_state.board.contains_color(first_pos, active_color) {
                let to_pos: Position = match get_next_position(&mut encoded_chars)? {
                    None => {
                        return Err(ChessError {
                            msg: format!("second position missing for {move_index} move for {active_color} after start position was {first_pos}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    Some(pos) => { pos }
                };
                FromTo::new(first_pos, to_pos)
            } else {
                let positions_with_figures_that_can_reach_target: Vec<Position> = get_positions_to_reach_target_from(first_pos, &game_state)?;
                let from_to: FromTo = match positions_with_figures_that_can_reach_target.len() {
                    0 => {
                        return Err(ChessError {
                            msg: format!("no position found that could reach {first_pos} in move {move_index} for {active_color}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    1 => { FromTo::new(positions_with_figures_that_can_reach_target[0], first_pos) }
                    _ => {
                        return Err(ChessError {
                            msg: format!("many position found that could reach {move_index} in move {active_color} for {first_pos}: {positions_with_figures_that_can_reach_target:?}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                };
                from_to
            };

            if game_state.looks_like_pawn_promotion_move(from_to) {
                let promotion_type: PromotionType = match encoded_chars.next() {
                    None => {
                        return Err(ChessError {
                            msg: format!("missing pawn promotion type at last decoded move {from_to}, one of 'Q', 'R', 'K' or 'B' was expected next depending on what figure the pawn should promoted to"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    Some(promotion_type_char) => {
                        match promotion_type_char.to_string().parse::<PromotionType>()  {
                            Ok(promotion_type) => {promotion_type}
                            Err(_) => {
                                return Err(ChessError {
                                    msg: format!("missing pawn promotion at decoded move {move_index}. {from_to}, one of 'Q', 'R', 'K' or 'B' was expected next depending on what figure the pawn should promoted to"),
                                    kind: ErrorKind::IllegalFormat,
                                });
                            }
                        }
                    }
                };
                Move::new_with_promotion(from_to, promotion_type)
            } else {
                Move::new(from_to)
            }
        };

        let (new_game_state, latest_move_data) = game_state.do_move(next_move);
        game_state = new_game_state;
        moves_played.push(latest_move_data);
        half_move_index = half_move_index + 1;
    }

    Ok(moves_played)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::compression::compress::compress;
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
    fn test_decompress(
        joined_moves: &str,
        expected_encoded_game_with_moves_separated_by_space: &str,
    ) {
        let moves: Vec<Move> = joined_moves.split(',').filter(|it| it.len() > 0).map(|it| it.trim().parse().unwrap()).collect();
        todo!("wait for test_compress to work")
        // let actual_encoded_game = compress(moves).unwrap();
        // let expected_encoded_game = expected_encoded_game_with_moves_separated_by_space.replace(' ', "");
        // assert_eq!(actual_encoded_game, expected_encoded_game);
    }
}