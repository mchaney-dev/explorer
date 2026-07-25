pub mod hex_color;
pub mod timestamp;

// Re-exported so callers can use `crate::models::shared::HexColor` /
// `crate::models::shared::Timestamp` as well as the fully-qualified paths.
pub use hex_color::HexColor;
pub use timestamp::Timestamp;
