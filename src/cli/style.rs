//! CLI styling utilities - Graphite-inspired color scheme
//!
//! Provides semantic styling via the [`Stylize`] trait with automatic
//! terminal color support detection (delegated to `owo-colors`).
//!
//! # Color Palette
//!
//! | Method       | Color  | Stream | Semantic Use                    |
//! |--------------|--------|--------|---------------------------------|
//! | `.accent()`  | Cyan   | stdout | Primary info: bookmarks, counts |
//! | `.success()` | Green  | stdout | Completion: checkmarks, done    |
//! | `.error()`   | Red    | stderr | Failures, error messages        |
//! | `.warn()`    | Yellow | stderr | Warnings, needs attention       |
//! | `.muted()`   | Dim    | stdout | Secondary: hints, metadata      |
//! | `.emphasis()`| Bold   | stdout | Headers, current action         |
//!
//! # Usage
//!
//! ```ignore
//! use crate::cli::style::Stylize;
//!
//! println!("{} done", check());
//! println!("Stack: {}", bookmark.accent());
//! eprintln!("{}", msg.error());
//! ```

use std::fmt::{self, Display};

pub use owo_colors::Stream;
use owo_colors::{OwoColorize, Style};

// ============================================================================
// Style definitions (single source of truth for color palette)
// ============================================================================

const ACCENT: Style = Style::new().cyan();
const SUCCESS: Style = Style::new().green();
const ERROR: Style = Style::new().red();
const WARN: Style = Style::new().yellow();
const MUTED: Style = Style::new().dimmed();
const EMPHASIS: Style = Style::new().bold();

// ============================================================================
// Styled wrapper
// ============================================================================

/// A value with semantic styling applied.
///
/// Implements [`Display`] to render with ANSI codes when supported.
/// Color support detection is handled by `owo-colors` (respects `NO_COLOR`,
/// `CLICOLOR`, `CLICOLOR_FORCE`, and TTY detection).
#[derive(Clone, Debug)]
pub struct Styled<T> {
    value: T,
    style: Style,
    stream: Stream,
}

impl<T> Styled<T> {
    const fn new(value: T, style: Style, stream: Stream) -> Self {
        Self {
            value,
            style,
            stream,
        }
    }

    /// Override to render for stderr stream detection.
    #[must_use]
    pub const fn for_stderr(mut self) -> Self {
        self.stream = Stream::Stderr;
        self
    }

    /// Override to render for stdout stream detection.
    #[must_use]
    pub const fn for_stdout(mut self) -> Self {
        self.stream = Stream::Stdout;
        self
    }
}

impl<T: Display> Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Single point where color detection + rendering happens.
        // owo-colors handles NO_COLOR, CLICOLOR, CLICOLOR_FORCE, TTY detection.
        write!(
            f,
            "{}",
            self.value
                .if_supports_color(self.stream, |v| v.style(self.style))
        )
    }
}

// ============================================================================
// Stylize trait
// ============================================================================

/// Extension trait for semantic terminal styling.
///
/// Automatically implemented for all [`Display`] types. Methods take `&self`
/// to avoid moving the value, allowing styling of borrowed data.
pub trait Stylize: Display {
    /// Accent color (cyan) for primary information.
    ///
    /// Use for: bookmark names, counts, URLs, commands
    fn accent(&self) -> Styled<&Self> {
        Styled::new(self, ACCENT, Stream::Stdout)
    }

    /// Success color (green) for completion states.
    ///
    /// Use for: checkmarks, "done", successful operations
    fn success(&self) -> Styled<&Self> {
        Styled::new(self, SUCCESS, Stream::Stdout)
    }

    /// Error color (red) for failures.
    ///
    /// Use for: error messages, failure indicators
    /// Default stream: stderr
    fn error(&self) -> Styled<&Self> {
        Styled::new(self, ERROR, Stream::Stderr)
    }

    /// Warning color (yellow) for attention-needed states.
    ///
    /// Use for: warnings, "needs push", uncommitted changes
    /// Default stream: stderr
    fn warn(&self) -> Styled<&Self> {
        Styled::new(self, WARN, Stream::Stderr)
    }

    /// Muted style (dim) for secondary information.
    ///
    /// Use for: hints, metadata, timestamps, de-emphasized text
    fn muted(&self) -> Styled<&Self> {
        Styled::new(self, MUTED, Stream::Stdout)
    }

    /// Emphasis style (bold) for important text.
    ///
    /// Use for: headers, current action, key information
    fn emphasis(&self) -> Styled<&Self> {
        Styled::new(self, EMPHASIS, Stream::Stdout)
    }
}

// Blanket implementation for all Display types
impl<T: Display + ?Sized> Stylize for T {}

// ============================================================================
// Symbols (Unicode)
// ============================================================================

/// Success checkmark
pub const CHECK: &str = "✓";

/// Error/failure cross
pub const CROSS: &str = "✗";

/// Arrow for steps/actions
pub const ARROW: &str = "→";

/// Bullet point for list items
pub const BULLET: &str = "○";

/// Current/active item marker
pub const CURRENT: &str = "@";

/// Vertical pipe for tree structure
pub const PIPE: &str = "│";

/// Up arrow for "needs push" indicator
pub const UP_ARROW: &str = "↑";

// ============================================================================
// Pre-styled symbol helpers
// ============================================================================

/// Green checkmark for success states.
#[inline]
pub const fn check() -> Styled<&'static str> {
    Styled::new(CHECK, SUCCESS, Stream::Stdout)
}

/// Red cross for error/failure states (renders to stderr by default).
#[inline]
pub const fn cross() -> Styled<&'static str> {
    Styled::new(CROSS, ERROR, Stream::Stderr)
}

/// Cyan arrow for action steps.
#[inline]
pub const fn arrow() -> Styled<&'static str> {
    Styled::new(ARROW, ACCENT, Stream::Stdout)
}

/// Dimmed bullet for list items.
#[inline]
pub const fn bullet() -> Styled<&'static str> {
    Styled::new(BULLET, MUTED, Stream::Stdout)
}

/// Dimmed pipe for tree structure.
#[inline]
pub const fn pipe() -> Styled<&'static str> {
    Styled::new(PIPE, MUTED, Stream::Stdout)
}

/// Yellow up-arrow for "needs push" indicator.
#[inline]
pub const fn up_arrow() -> Styled<&'static str> {
    Styled::new(UP_ARROW, WARN, Stream::Stdout)
}

// ============================================================================
// Hyperlinks (OSC 8)
// ============================================================================

/// Convert owo-colors Stream to supports-hyperlinks Stream
const fn to_hyperlink_stream(stream: Stream) -> supports_hyperlinks::Stream {
    match stream {
        Stream::Stdout => supports_hyperlinks::Stream::Stdout,
        Stream::Stderr => supports_hyperlinks::Stream::Stderr,
    }
}

/// Create a clickable hyperlink showing the URL itself.
///
/// Falls back to plain URL text in terminals that don't support OSC 8 hyperlinks.
pub fn hyperlink_url(stream: Stream, url: &str) -> String {
    if supports_hyperlinks::on(to_hyperlink_stream(stream)) {
        terminal_link::Link::new(url, url).to_string()
    } else {
        url.to_string()
    }
}

// ============================================================================
// Spinner Styles
// ============================================================================

use indicatif::ProgressStyle;
use std::sync::OnceLock;

/// Default spinner style - cyan dots.
///
/// Template validated once on first call via `OnceLock`.
pub fn spinner_style() -> ProgressStyle {
    static STYLE: OnceLock<ProgressStyle> = OnceLock::new();
    STYLE
        .get_or_init(|| {
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("hardcoded spinner template is valid")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        })
        .clone()
}
