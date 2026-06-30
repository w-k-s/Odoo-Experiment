pub fn new_id(prefix: &str) -> String {
    format!("{}-{}", prefix, nanoid::nanoid!(10))
}
