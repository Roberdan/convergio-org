//! Input validation helpers for convergio-org endpoints.
//!
//! Centralises length limits, format checks, and enum validation so that
//! every handler applies consistent rules.

/// Maximum length for identifiers (org_id, agent name, skill name).
pub const MAX_ID_LEN: usize = 128;

/// Maximum length for short text fields (mission, role, title, service_name).
pub const MAX_SHORT_TEXT: usize = 1_000;

/// Maximum length for long text fields (question, reasoning, message body).
pub const MAX_LONG_TEXT: usize = 10_000;

/// Maximum length for very long text fields (objectives, knowledge content).
pub const MAX_VERY_LONG_TEXT: usize = 50_000;

/// Allowed severity values for notifications.
const VALID_SEVERITIES: &[&str] = &["info", "warning", "error", "success"];

/// Validate an identifier (org_id, agent name, etc.).
/// Must be non-empty, ≤ `MAX_ID_LEN`, and contain only safe characters.
pub fn validate_id(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field_name} must not be empty"));
    }
    if value.len() > MAX_ID_LEN {
        return Err(format!(
            "{field_name} too long ({} chars, max {MAX_ID_LEN})",
            value.len()
        ));
    }
    // Allow alphanumeric, hyphens, underscores, dots, colons
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c))
    {
        return Err(format!(
            "{field_name} contains invalid characters (allowed: alphanumeric, -, _, ., :)"
        ));
    }
    Ok(())
}

/// Validate a short text field.
pub fn validate_short_text(value: &str, field_name: &str) -> Result<(), String> {
    if value.len() > MAX_SHORT_TEXT {
        return Err(format!(
            "{field_name} too long ({} chars, max {MAX_SHORT_TEXT})",
            value.len()
        ));
    }
    Ok(())
}

/// Validate a long text field (questions, reasoning).
pub fn validate_long_text(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field_name} must not be empty"));
    }
    if value.len() > MAX_LONG_TEXT {
        return Err(format!(
            "{field_name} too long ({} chars, max {MAX_LONG_TEXT})",
            value.len()
        ));
    }
    Ok(())
}

/// Validate notification severity.
pub fn validate_severity(severity: &str) -> Result<(), String> {
    if VALID_SEVERITIES.contains(&severity) {
        Ok(())
    } else {
        Err(format!(
            "invalid severity '{severity}', must be one of: {}",
            VALID_SEVERITIES.join(", ")
        ))
    }
}

/// Validate confidence is in [0.0, 1.0].
pub fn validate_confidence(value: f64) -> Result<(), String> {
    if !(0.0..=1.0).contains(&value) {
        return Err(format!(
            "confidence must be between 0.0 and 1.0, got {value}"
        ));
    }
    Ok(())
}

/// Validate a query limit parameter.
pub fn validate_limit(limit: u32, max: u32) -> u32 {
    limit.min(max).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ids() {
        assert!(validate_id("my-org", "org_id").is_ok());
        assert!(validate_id("org_123", "org_id").is_ok());
        assert!(validate_id("a.b:c", "org_id").is_ok());
    }

    #[test]
    fn invalid_ids() {
        assert!(validate_id("", "org_id").is_err());
        assert!(validate_id("a b", "org_id").is_err());
        assert!(validate_id("a/b", "org_id").is_err());
        assert!(validate_id(&"x".repeat(200), "org_id").is_err());
    }

    #[test]
    fn severity_validation() {
        assert!(validate_severity("info").is_ok());
        assert!(validate_severity("error").is_ok());
        assert!(validate_severity("critical").is_err());
        assert!(validate_severity("").is_err());
    }

    #[test]
    fn confidence_validation() {
        assert!(validate_confidence(0.0).is_ok());
        assert!(validate_confidence(1.0).is_ok());
        assert!(validate_confidence(0.5).is_ok());
        assert!(validate_confidence(-0.1).is_err());
        assert!(validate_confidence(1.1).is_err());
    }

    #[test]
    fn limit_clamping() {
        assert_eq!(validate_limit(0, 200), 1);
        assert_eq!(validate_limit(50, 200), 50);
        assert_eq!(validate_limit(500, 200), 200);
    }

    #[test]
    fn long_text_validation() {
        assert!(validate_long_text("hello", "q").is_ok());
        assert!(validate_long_text("", "q").is_err());
        assert!(validate_long_text(&"x".repeat(MAX_LONG_TEXT + 1), "q").is_err());
    }
}
