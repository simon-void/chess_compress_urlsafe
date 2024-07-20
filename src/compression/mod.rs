pub mod compress;
pub mod decompress;
mod base64;

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use rstest_reuse::{self, *};
    use crate::base::a_move::{Move, MoveData};
    use crate::base::a_move::MoveType::PawnPromotion;
    use crate::base::util::tests::parse_to_vec;
    use crate::base::util::vec_to_str;
    use crate::compression::compress::compress;
    use crate::compression::decompress::{decompress, PositionData};

    fn remove_space(s: &str) -> String {
        s.replace(' ', "")
    }

    fn extract_given_move(vec_of_move_data: Vec<MoveData>) -> Vec<Move> {
        vec_of_move_data.iter().map(|it| {
            let from_to = it.given_from_to;
            if let PawnPromotion { promoted_to: promotion_type } = it.move_type {
                Move::new_with_promotion(from_to, promotion_type)
            } else {
                Move::new(from_to)
            }
        }).collect()
    }

    #[template]
    #[rstest]
    #[case("", "")]       //    | "no moves -> empty encoded String
    #[case("c2c3", "KS")] // KS | destination not unique target -> encoding needs two chars
    #[case("c2c4", "a")]  // Ka | destination is unique target -> encoding needs one char
    #[case("a2a4, h7h6, a4a5, b7b5, a5b6, h6h5, b6c7, h5h4, g2g3, h4g3, c7d8Q", "Y 3v g h p n y f W W 7Q")] // IY 3v Yg xh gp vn py nf OW fW y7Q | tests all pawn moves single-step, double-step, diagonal-capture, en-passant & promotion
    #[case("d2d3, g7g6, c1e3, f8g7, b1c3, g8f6, d1d2, e8h8, e1a1", "T u CU 2 BS -t DL 8_ EA")]              // LT 2u CU 92 BS -t DL 8_ EA        | tests king- & queen-side castling
    fn compress_decompress_cases(#[case] decoded_moves: &str, #[case] encoded_moves_seperated_by_space: &str) {}

    #[apply(compress_decompress_cases)]
    fn test_compress(decoded_moves: &str, encoded_moves_seperated_by_space: &str) {
        let actual_encoded_game: String = {
            let given_moves: Vec<Move> = parse_to_vec(&decoded_moves, ",").unwrap();
            compress(given_moves).unwrap()
        };
        let expected_encoded_game: String = remove_space(encoded_moves_seperated_by_space);
        assert_eq!(actual_encoded_game, expected_encoded_game);
    }

    #[apply(compress_decompress_cases)]
    fn test_decompress(decoded_moves: &str, encoded_moves_seperated_by_space: &str) {
        let actual_decoded_moves = {
            let given_encoded_game = remove_space(encoded_moves_seperated_by_space);
            let (_positions_data, moves_data): (Vec<PositionData>, Vec<MoveData>) = decompress(given_encoded_game.as_str()).unwrap();
            let given_moves: Vec<Move> = extract_given_move(moves_data);
            vec_to_str(&given_moves, ",")
        };
        let expected_decoded_moves = format!("[{}]", remove_space(decoded_moves));
        assert_eq!(expected_decoded_moves, actual_decoded_moves);
    }
}
