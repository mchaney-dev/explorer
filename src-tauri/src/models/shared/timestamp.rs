use chrono::{DateTime, Utc};

/// The timestamp type used throughout the data models.
///
/// Aliased (like [`crate::models::shared::HexColor`]) so the concrete
/// representation lives in one place. It serializes to an RFC 3339 string for
/// both the JSON sent to the frontend and ISO-8601 TEXT storage in SQLite —
/// every mainstream Rust SQLite driver maps `DateTime<Utc>` to a TEXT column by
/// default. To move to INTEGER epoch storage or a custom encoding later, swap
/// this alias for a newtype and add the driver's `ToSql`/`FromSql` impls here.
pub type Timestamp = DateTime<Utc>;
