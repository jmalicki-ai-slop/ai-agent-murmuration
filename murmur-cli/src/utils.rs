//! Utility functions shared across CLI commands

/// Choose between emoji and ASCII alternative based on no_emoji flag
///
/// # Arguments
/// * `no_emoji` - If true, return the ASCII alternative
/// * `emoji_char` - The emoji character to use
/// * `ascii_alt` - The ASCII alternative to use when no_emoji is true
///
/// # Example
/// ```
/// let output = emoji(false, "🔥", "!!");
/// assert_eq!(output, "🔥");
///
/// let output = emoji(true, "🔥", "!!");
/// assert_eq!(output, "!!");
/// ```
pub fn emoji<'a>(no_emoji: bool, emoji_char: &'a str, ascii_alt: &'a str) -> &'a str {
    if no_emoji {
        ascii_alt
    } else {
        emoji_char
    }
}
