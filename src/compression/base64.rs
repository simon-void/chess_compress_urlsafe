use std::sync::OnceLock;
use regex::Regex;
use crate::base::{ChessError, ErrorKind, Position};
// using url safe base 64 encoding without the padding character since it's not needed
// since a chessboard has 64 fields so the index of a field takes exactly 6bits or one base64 value.
//
// https://datatracker.ietf.org/doc/html/rfc4648#section-5
//
// 0 A            17 R            34 i            51 z
// 1 B            18 S            35 j            52 0
// 2 C            19 T            36 k            53 1
// 3 D            20 U            37 l            54 2
// 4 E            21 V            38 m            55 3
// 5 F            22 W            39 n            56 4
// 6 G            23 X            40 o            57 5
// 7 H            24 Y            41 p            58 6
// 8 I            25 Z            42 q            59 7
// 9 J            26 a            43 r            60 8
//10 K            27 b            44 s            61 9
//11 L            28 c            45 t            62 - (minus)
//12 M            29 d            46 u            63 _ (underline)
//13 N            30 e            47 v
//14 O            31 f            48 w
//15 P            32 g            49 x
//16 Q            33 h            50 y


pub fn decode_base64(character: char) -> Result<Position, ChessError> {
    let decoded: i8 = match character {
        'A' => { 0 }
        'B' => { 1 }
        'C' => { 2 }
        'D' => { 3 }
        'E' => { 4 }
        'F' => { 5 }
        'G' => { 6 }
        'H' => { 7 }
        'I' => { 8 }
        'J' => { 9 }
        'K' => { 10 }
        'L' => { 11 }
        'M' => { 12 }
        'N' => { 13 }
        'O' => { 14 }
        'P' => { 15 }
        'Q' => { 16 }
        'R' => { 17 }
        'S' => { 18 }
        'T' => { 19 }
        'U' => { 20 }
        'V' => { 21 }
        'W' => { 22 }
        'X' => { 23 }
        'Y' => { 24 }
        'Z' => { 25 }
        'a' => { 26 }
        'b' => { 27 }
        'c' => { 28 }
        'd' => { 29 }
        'e' => { 30 }
        'f' => { 31 }
        'g' => { 32 }
        'h' => { 33 }
        'i' => { 34 }
        'j' => { 35 }
        'k' => { 36 }
        'l' => { 37 }
        'm' => { 38 }
        'n' => { 39 }
        'o' => { 40 }
        'p' => { 41 }
        'q' => { 42 }
        'r' => { 43 }
        's' => { 44 }
        't' => { 45 }
        'u' => { 46 }
        'v' => { 47 }
        'w' => { 48 }
        'x' => { 49 }
        'y' => { 50 }
        'z' => { 51 }
        '0' => { 52 }
        '1' => { 53 }
        '2' => { 54 }
        '3' => { 55 }
        '4' => { 56 }
        '5' => { 57 }
        '6' => { 58 }
        '7' => { 59 }
        '8' => { 60 }
        '9' => { 61 }
        '-' => { 62 }
        '_' => { 63 }
        _ => {
            return Err(ChessError {
                msg: format!("not a url safe base64 char: {character}"),
                kind: ErrorKind::IllegalFormat
            })
        }
    };
    let column_index = decoded % 8;
    let row_index = decoded / 8;
    Ok(Position::new_unchecked(column_index, row_index))
}

pub fn encode_base64(position: Position) -> char {
    static ONCE: OnceLock<[char; 64]> = OnceLock::new();
    let url_safe_base64_chars: &[char; 64] = ONCE.get_or_init(|| {
        ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_']
    });
    url_safe_base64_chars[position.index]
}

pub fn assert_is_url_safe_base64(str: &str) -> Result<(), ChessError> {
    // OnceCell is not thread-safe! for that use OnceLock
    static ONCE: OnceLock<Regex> = OnceLock::new();
    let url_safe_base64_regex: &Regex = ONCE.get_or_init(|| {
        Regex::new(r"^([a-z]|[A-Z]|[0-9]|-|_)*$").unwrap()
    });

    if url_safe_base64_regex.is_match(str) {
        Ok(())
    } else {
        Err(ChessError {
            msg: "value contained characters that are not allowed for a url-safe base64 encoded String. following characters are allowed: a-z, A-Z, 0-1, -, _".to_string(),
            kind: ErrorKind::IllegalFormat
        })
    }
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::compression::base64::assert_is_url_safe_base64;

    #[rstest(
        value, expected_is_legal,
        case("", true),
        case("h", true),
        case("M", true),
        case("5", true),
        case("abcxyz", true),
        case("ABCXYZ", true),
        case("0189", true),
        case("-", true),
        case("_", true),
        case("_k-sA1Y0", true),
        case("55--__ffYY", true),
        case("=", false),
        case("+", false),
        case("&", false),
        case("?", false),
        case(" ", false),
        case(" asldkf9", false),
        case(" KJD_", false),
        case("asldkf9+", false),
        case("asl=dkf9", false),
        case("KJD_?", false),
        case("^", false),
        case("$", false),
        case("^$", false),
        case("^fI6$", false),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_legal_url_safe_base64_values(value: &str, expected_is_legal: bool) {
        let base64_result = assert_is_url_safe_base64(value);
        match base64_result {
            Ok(_) => {
                if !expected_is_legal {
                    panic!("value {} wasn't recognized as illegal base64", value)
                }
            },
            Err(_) => {
                if expected_is_legal {
                    panic!("value {} wasn't recognized as legal base64", value)
                }
            }
        };
    }
}