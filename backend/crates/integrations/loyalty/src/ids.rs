pub fn new_id(prefix: &str) -> String {
    format!("{}-{}", prefix, nanoid::nanoid!(10))
}

pub fn new_session_code() -> String {
    nanoid::nanoid!(8)
}
