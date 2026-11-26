//! Checks shared by the endpoints.

use crate::error::ApiError;

/// Longest name of anything a user can name.
pub const MAX_NAME_LENGTH: usize = 100;

/// Trims a name and checks that it is neither empty nor overlong.
pub fn clean_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid("Der Name fehlt."));
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::invalid("Der Name ist zu lang."));
    }
    Ok(name.to_owned())
}

/// Whether two names are the same for a user: case and surrounding spaces
/// do not matter.
pub fn same_name(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_limited() {
        assert_eq!(clean_name("  7a ").unwrap(), "7a");
        assert!(clean_name("   ").is_err());
        assert!(clean_name(&"x".repeat(MAX_NAME_LENGTH)).is_ok());
        assert!(clean_name(&"x".repeat(MAX_NAME_LENGTH + 1)).is_err());
    }

    #[test]
    fn same_name_ignores_case_and_spaces() {
        assert!(same_name("7a", " 7A "));
        assert!(same_name("Technik", "technik"));
        assert!(!same_name("7a", "7b"));
    }
}
