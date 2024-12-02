#[cfg(not(feature = "with_integrity"))]
pub fn get_integrity_hash() -> String {
    String::new()
}
