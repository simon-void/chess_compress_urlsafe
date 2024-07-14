use crate::base::a_move::Move;
use crate::compression::base64::encode_base64;
use crate::figure::functions::is_reachable_by::get_positions_to_reach_target_from;
use crate::base::color::Color;
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::base::util::vec_to_str;
use crate::game::game_state::GameState;

pub fn compress(moves: Vec<Move>) -> Result<String, ChessError> {
    let mut game_state = GameState::classic();
    let mut encoded_moves = String::with_capacity(moves.len()*2);

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
                    let err_msg = {
                        let moving_figure_type = match &game_state.board.get_figure(next_move.from_to.from).map(|figure|figure.fig_type) {
                            None => {"Empty".to_string()}
                            Some(figure_type) => {format!("{figure_type:?}")}
                        };
                        let mut msg = match active_color {
                            Color::White => format!("move {move_nr}. {next_move} .. "),
                            Color::Black => format!("move {move_nr}. .. {next_move} "),
                        };
                        msg.push_str(format!("is illegal since you can't go there with a {moving_figure_type}. {} is only reachable from {}", next_move.from_to.from, vec_to_str(&positions_with_figures_that_can_reach_target, ", ")).as_str());
                        msg
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

// Tests are in compression/mod.rs
