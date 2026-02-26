//! Audit log — records every interaction through Omega.

use omega_core::error::OmegaError;
use sqlx::SqlitePool;
use tracing::debug;
use uuid::Uuid;

/// An entry to write to the audit log.
pub struct AuditEntry {
    pub channel: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub input_text: String,
    pub output_text: Option<String>,
    pub provider_used: Option<String>,
    pub model: Option<String>,
    pub processing_ms: Option<i64>,
    pub status: AuditStatus,
    pub denial_reason: Option<String>,
}

/// Status of an audited interaction.
pub enum AuditStatus {
    Ok,
    Error,
    Denied,
}

impl AuditStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Denied => "denied",
        }
    }
}

/// Audit logger backed by SQLite.
#[derive(Clone)]
pub struct AuditLogger {
    pool: SqlitePool,
}

impl AuditLogger {
    /// Create a new audit logger sharing the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Write an entry to the audit log.
    pub async fn log(&self, entry: &AuditEntry) -> Result<(), OmegaError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO audit_log \
             (id, channel, sender_id, sender_name, input_text, output_text, \
              provider_used, model, processing_ms, status, denial_reason) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&entry.channel)
        .bind(&entry.sender_id)
        .bind(&entry.sender_name)
        .bind(&entry.input_text)
        .bind(&entry.output_text)
        .bind(&entry.provider_used)
        .bind(&entry.model)
        .bind(entry.processing_ms)
        .bind(entry.status.as_str())
        .bind(&entry.denial_reason)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("audit log write failed: {e}")))?;

        debug!(
            "audit: {} {} [{}] {}",
            entry.channel,
            entry.sender_id,
            entry.status.as_str(),
            truncate(&entry.input_text, 80)
        );

        Ok(())
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..s.floor_char_boundary(max)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello");
    }

    #[test]
    fn test_truncate_multibyte() {
        // "Привет мир!" in UTF-8: each Cyrillic letter is 2 bytes, space is 1 byte
        // П(2) р(2) и(2) в(2) е(2) т(2) ' '(1) м(2) и(2) р(2) !(1) = 21 bytes
        let s = "\u{041f}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442} \u{043c}\u{0438}\u{0440}!";
        // byte 5 falls inside the 3rd character (и starts at byte 4, ends at byte 6)
        let result = truncate(s, 5);
        // Should NOT panic; should truncate at a valid char boundary
        assert!(!result.is_empty());
    }

    #[test]
    fn test_truncate_emoji() {
        // "Hi 🎉 there": H(1) i(1) ' '(1) 🎉(4) ' '(1) ...
        // byte 4 falls inside the 🎉 emoji (bytes 3..7)
        let s = "Hi \u{1f389} there";
        let result = truncate(s, 4);
        // Should NOT panic; should truncate at a valid char boundary
        assert!(!result.is_empty());
    }
}
