# RFC: CLI Styling System

**Status:** Implemented
**Author:** Dillon Mulroy
**Date:** 2026-01-03
**Scope:** `src/cli/`

---

## Summary

Add a unified styling system to CLI output via a `Stylize` extension trait with semantic methods (`.accent()`, `.success()`, `.error()`, etc.). Includes Unicode symbols, progress spinners, and clickable terminal hyperlinks. Color detection and ANSI rendering delegated to `owo-colors`.

---

## Motivation

### Problem Statement

Current CLI output is plain text with no visual differentiation:
- Success/error states indistinguishable
- Important info (bookmark names, counts, PR numbers) blends with prose
- No progress indication during network operations
- PR URLs not clickable in supported terminals

### Goals

- Semantic styling API that enforces consistent color palette
- Visual hierarchy via bold/dim/colored text
- Progress spinners for async operations (auth, fetch)
- Clickable hyperlinks for PR URLs
- Graceful degradation in non-color terminals

### Non-Goals

- Custom themes or user-configurable colors
- Windows Console API support (rely on ANSI detection)
- Animation beyond spinners

---

## Design

### Stylize Extension Trait

Core abstraction: blanket-implemented trait on all `Display` types.

```rust
pub trait Stylize: Display {
    fn accent(&self) -> Styled<&Self>;    // cyan, stdout
    fn success(&self) -> Styled<&Self>;   // green, stdout
    fn error(&self) -> Styled<&Self>;     // red, stderr
    fn warn(&self) -> Styled<&Self>;      // yellow, stderr
    fn muted(&self) -> Styled<&Self>;     // dim, stdout
    fn emphasis(&self) -> Styled<&Self>;  // bold, stdout
}

impl<T: Display + ?Sized> Stylize for T {}
```

**Key design decisions:**

1. **`&self` not `self`:** Avoids moving owned types like `String`. Allows `bookmark.accent()` without cloning.

2. **Semantic names:** `accent()` not `cyan()`. Enforces palette consistency - prevents ad-hoc color choices.

3. **Stream defaults:**
   - `.error()` and `.warn()` default to stderr (Unix convention)
   - All others default to stdout
   - `.for_stderr()` / `.for_stdout()` override when needed

4. **Delegate to owo-colors:** Color detection and ANSI rendering use `owo-colors::if_supports_color()` in a single location (`Styled::fmt()`), avoiding scattered conditional color logic.

### Color Palette (Graphite-inspired)

Defined as `const` values using `owo_colors::Style`:

```rust
const ACCENT: Style = Style::new().cyan();
const SUCCESS: Style = Style::new().green();
const ERROR: Style = Style::new().red();
const WARN: Style = Style::new().yellow();
const MUTED: Style = Style::new().dimmed();
const EMPHASIS: Style = Style::new().bold();
```

| Method | Color | Stream | Semantic Use |
|--------|-------|--------|--------------|
| `.accent()` | Cyan | stdout | Primary info: bookmarks, counts, URLs |
| `.success()` | Green | stdout | Completion: checkmarks, "done" |
| `.error()` | Red | stderr | Failures, error messages |
| `.warn()` | Yellow | stderr | Warnings, needs attention |
| `.muted()` | Dim | stdout | Secondary: hints, metadata |
| `.emphasis()` | Bold | stdout | Headers, current action |

### Styled Wrapper

Thin wrapper that captures value, style, and target stream:

```rust
pub struct Styled<T> {
    value: T,
    style: Style,
    stream: Stream,
}

impl<T: Display> Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Single point where color detection happens
        write!(f, "{}", self.value.if_supports_color(self.stream, |v| v.style(self.style)))
    }
}
```

Color support detection is handled entirely by `owo-colors` (respects `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, TTY detection).

### Symbol Constants & Pre-styled Helpers

```rust
// Raw constants
pub const CHECK: &str = "✓";
pub const CROSS: &str = "✗";
pub const ARROW: &str = "→";
pub const BULLET: &str = "○";
pub const CURRENT: &str = "@";
pub const PIPE: &str = "│";
pub const UP_ARROW: &str = "↑";

// Pre-styled helpers (const fn)
pub const fn check() -> Styled<&'static str>;    // green checkmark
pub const fn cross() -> Styled<&'static str>;    // red cross
pub const fn arrow() -> Styled<&'static str>;    // cyan arrow
pub const fn bullet() -> Styled<&'static str>;   // dim bullet
pub const fn pipe() -> Styled<&'static str>;     // dim pipe
pub const fn up_arrow() -> Styled<&'static str>; // yellow up-arrow
```

### Usage

```rust
use crate::cli::style::{check, Stylize};

// Semantic styling
println!("{}", "Header".emphasis());
println!("{}", bookmark.accent());
println!("{}", "done".success());

// Pre-styled symbols
println!("{} Pushed {}", check(), bookmark.accent());

// Errors (uses stderr color detection)
eprintln!("{}", msg.error());

// Override stream when needed
println!("{}", cross().for_stderr());
```

### Progress Spinners

```rust
let spinner = ProgressBar::new_spinner();
spinner.set_style(spinner_style());
spinner.set_message("Fetching...");
spinner.enable_steady_tick(Duration::from_millis(80));
// ... async work ...
spinner.finish_and_clear();
```

### Hyperlinks (OSC 8)

```rust
println!("PR: {}", hyperlink_url(Stream::Stdout, &pr.html_url));
// Supported terminals: clickable link
// Fallback: plain URL text
```

---

## Changes Summary

| File | Lines | Description |
|------|-------|-------------|
| `Cargo.toml` | +6 | Add `owo-colors`, `anstream`, `indicatif`, `terminal-link`, `supports-hyperlinks` |
| `src/cli/style.rs` | ~260 | `Stylize` trait, `Styled<T>` wrapper, const styles, symbols, hyperlinks, spinner |
| `src/cli/mod.rs` | +1 | Export `style` module |
| `src/cli/analyze.rs` | ~50 | Style stack visualization output |
| `src/cli/auth.rs` | ~50 | Spinners + styled auth output |
| `src/cli/progress.rs` | ~50 | Style progress callbacks |
| `src/cli/submit.rs` | ~50 | Style submission output |
| `src/cli/sync.rs` | ~50 | Style sync output + fetch spinner |

---

## Trade-offs and Alternatives

### Alternative: `colored` crate

**Rejected:** `owo-colors` is more actively maintained, const-friendly API, better `anstream` integration.

### Alternative: Macros

**Rejected:** Macros obscure types, harder to debug, worse IDE support than method chains.

### Alternative: Raw color methods (`cyan()`, `green()`)

**Rejected:** Exposes implementation, allows inconsistent palette. Semantic names enforce consistency.

### Alternative: Hand-rolled ANSI codes + color detection

**Rejected:** Reimplements what `owo-colors` already provides. Using `if_supports_color()` delegates env var parsing (`NO_COLOR`, `CLICOLOR`, etc.) and TTY detection to battle-tested library code.

### Trade-off: No style composition

Single semantic style per value (no `.emphasis().accent()` chaining). Simplifies API, covers all current use cases.

### Trade-off: Extra dependencies

5 new crates. Accepted because:
- All well-maintained, widely used
- Provide correct terminal detection across platforms
- Total size impact minimal for CLI binary

---

## Testing Strategy

| Test Type | Location | Validates |
|-----------|----------|-----------|
| Manual | N/A | Visual inspection in iTerm2, Terminal.app, VS Code |
| CI | GitHub Actions | `cargo clippy`, `cargo build`, `cargo test` |

No automated tests for styling - color output is visual/UX concern. Existing functional tests continue to validate command behavior.

---

## Migration & Compatibility

### API Changes

None - internal CLI module only. Library crate (`jj_ryu`) unchanged.

### Breaking Changes

None. Output text content unchanged, only formatting added. Non-color terminals see plain text.

---

## Security Considerations

None - styling is output-only, no new inputs or auth changes.

---

## Open Questions

1. **Windows terminal support?** - Relies on `owo-colors` detection + ANSI. Needs testing on Windows Terminal vs legacy cmd.exe.

---

## Conclusion

Adds professional CLI polish with semantic styling API. `Stylize` trait enforces consistent visual language. Pre-styled symbol helpers cover common patterns. Color detection delegated to `owo-colors` via single `if_supports_color()` call. Graceful fallback for unsupported terminals. No breaking changes.
