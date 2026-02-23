//! Reward-based learning: raw outcomes (working memory) and distilled lessons (long-term memory).
//!
//! All functions accept a `project` parameter for project-scoped isolation.
//! Empty string `""` = general OMEGA (no project).

use super::Store;
use omega_core::error::OmegaError;

impl Store {
    /// Store a raw outcome from a REWARD marker.
    pub async fn store_outcome(
        &self,
        sender_id: &str,
        domain: &str,
        score: i32,
        lesson: &str,
        source: &str,
        project: &str,
    ) -> Result<(), OmegaError> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO outcomes (id, sender_id, domain, score, lesson, source, project) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(sender_id)
        .bind(domain)
        .bind(score)
        .bind(lesson)
        .bind(source)
        .bind(project)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("store outcome: {e}")))?;
        Ok(())
    }

    /// Get recent outcomes for a sender (for regular conversation prompt injection).
    ///
    /// When `project` is Some, returns only outcomes for that project.
    /// When `project` is None, returns all outcomes (general behavior).
    /// Returns `(score, domain, lesson, timestamp)` ordered newest first.
    pub async fn get_recent_outcomes(
        &self,
        sender_id: &str,
        limit: i64,
        project: Option<&str>,
    ) -> Result<Vec<(i32, String, String, String)>, OmegaError> {
        let rows: Vec<(i32, String, String, String)> = match project {
            Some(p) => {
                sqlx::query_as(
                    "SELECT score, domain, lesson, timestamp FROM outcomes \
                     WHERE sender_id = ? AND project = ? ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(sender_id)
                .bind(p)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as(
                    "SELECT score, domain, lesson, timestamp FROM outcomes \
                     WHERE sender_id = ? ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(sender_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| OmegaError::Memory(format!("get recent outcomes: {e}")))?;
        Ok(rows)
    }

    /// Get recent outcomes across all users (for heartbeat enrichment).
    ///
    /// When `project` is Some, returns only outcomes for that project.
    /// When `project` is None, returns all outcomes.
    /// Returns `(score, domain, lesson, timestamp)` within the last N hours.
    pub async fn get_all_recent_outcomes(
        &self,
        hours: i64,
        limit: i64,
        project: Option<&str>,
    ) -> Result<Vec<(i32, String, String, String)>, OmegaError> {
        let rows: Vec<(i32, String, String, String)> = match project {
            Some(p) => {
                sqlx::query_as(
                    "SELECT score, domain, lesson, timestamp FROM outcomes \
                     WHERE datetime(timestamp) >= datetime('now', ? || ' hours') \
                     AND project = ? \
                     ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(-hours)
                .bind(p)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as(
                    "SELECT score, domain, lesson, timestamp FROM outcomes \
                     WHERE datetime(timestamp) >= datetime('now', ? || ' hours') \
                     ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(-hours)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| OmegaError::Memory(format!("get all recent outcomes: {e}")))?;
        Ok(rows)
    }

    /// Store or update a distilled lesson (upsert by sender_id + domain + project).
    ///
    /// If a lesson already exists for this domain+project, the rule is replaced and
    /// occurrences is incremented. Otherwise a new lesson is created.
    pub async fn store_lesson(
        &self,
        sender_id: &str,
        domain: &str,
        rule: &str,
        project: &str,
    ) -> Result<(), OmegaError> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO lessons (id, sender_id, domain, rule, project) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(sender_id, domain, project) DO UPDATE SET \
             rule = excluded.rule, \
             occurrences = occurrences + 1, \
             updated_at = datetime('now')",
        )
        .bind(&id)
        .bind(sender_id)
        .bind(domain)
        .bind(rule)
        .bind(project)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("store lesson: {e}")))?;
        Ok(())
    }

    /// Get lessons for a sender.
    ///
    /// When `project` is Some, returns project-specific lessons first, then general.
    /// When `project` is None, returns general lessons only (project = '').
    /// Returns `(domain, rule, project)` ordered by most-updated first.
    pub async fn get_lessons(
        &self,
        sender_id: &str,
        project: Option<&str>,
    ) -> Result<Vec<(String, String, String)>, OmegaError> {
        let rows: Vec<(String, String, String)> = match project {
            Some(p) => {
                // Project-specific first (sorted by project DESC so non-empty comes first),
                // then general. Both ordered by updated_at DESC within each group.
                sqlx::query_as(
                    "SELECT domain, rule, project FROM lessons \
                     WHERE sender_id = ? AND (project = ? OR project = '') \
                     ORDER BY CASE WHEN project = ? THEN 0 ELSE 1 END, updated_at DESC",
                )
                .bind(sender_id)
                .bind(p)
                .bind(p)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as(
                    "SELECT domain, rule, project FROM lessons \
                     WHERE sender_id = ? AND project = '' ORDER BY updated_at DESC",
                )
                .bind(sender_id)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| OmegaError::Memory(format!("get lessons: {e}")))?;
        Ok(rows)
    }

    /// Get all lessons across all users (for heartbeat enrichment).
    ///
    /// When `project` is Some, returns project-specific + general lessons.
    /// When `project` is None, returns all lessons.
    /// Returns `(domain, rule, project)` ordered by most-updated first.
    pub async fn get_all_lessons(
        &self,
        project: Option<&str>,
    ) -> Result<Vec<(String, String, String)>, OmegaError> {
        let rows: Vec<(String, String, String)> = match project {
            Some(p) => {
                sqlx::query_as(
                    "SELECT domain, rule, project FROM lessons \
                     WHERE project = ? OR project = '' \
                     ORDER BY CASE WHEN project = ? THEN 0 ELSE 1 END, updated_at DESC",
                )
                .bind(p)
                .bind(p)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as("SELECT domain, rule, project FROM lessons ORDER BY updated_at DESC")
                    .fetch_all(&self.pool)
                    .await
            }
        }
        .map_err(|e| OmegaError::Memory(format!("get all lessons: {e}")))?;
        Ok(rows)
    }
}
