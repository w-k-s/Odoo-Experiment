use nanoid::nanoid;

/// Unambiguous uppercase alphabet (no 0/O/1/I) for human-readable codes.
const CODE_ALPHABET: [char; 32] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L',
    'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Prefixed entity id, e.g. `prog_V1StGXR8Z5jd`.
pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", nanoid!(12))
}

/// Short, scannable session code presented at the till, e.g. `LOY-7F3A9K`.
pub fn new_session_code() -> String {
    format!("LOY-{}", nanoid!(6, &CODE_ALPHABET))
}
