# CLI Styling & Terminal Control

Console styling transforms a basic terminal output into a professional, readable experience. This covers colors, escape codes, terminal queries, hyperlinks, images, and raw mode.

## Console Colors in Rust

Colorizing console output in Rust is a common task, and the ecosystem offers several excellent crates ranging from low-level control to high-level convenience.

### Color Library Comparison

| Crate | Style | Learning Curve | Best Use Case |
| --- | --- | --- | --- |
| **`colored`** | `String` extensions | Very Easy | Prototyping & small scripts |
| **`termcolor`** | Buffer-based | Moderate | Professional, cross-platform tools |
| **`owo-colors`** | Wrapper-based | Easy | Performance-critical applications |
| **`nu-ansi-term`** | Struct-based | Easy | Complex styling and layout |

### Colored (Easiest to Use)

The quickest, most readable syntax. Uses an extension trait to add coloring methods directly to strings.

```rust
use colored::*;

fn main() {
    println!("{} {} {}",
        "Error:".red().bold(),
        "Something went wrong".bright_yellow(),
        " (code: 404)".black().on_white()
    );
}
```

**Pros:** Extremely intuitive; no complex setup
**Best for:** Small tools, scripts, and quick debugging

### Termcolor (Cross-Platform Standard)

The industry standard for professional CLI tools, especially on Windows. Used by the Rust compiler and tools like `ripgrep`.

```rust
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

let mut stdout = StandardStream::stdout(ColorChoice::Always);
stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
writeln!(&mut stdout, "Error")?;
stdout.reset()?;
```

**Pros:** High performance, very reliable, handles `NO_COLOR` environment variables
**Cons:** More boilerplate code compared to `colored`

### Owo-colors (Zero-Cost & Type-Safe)

Modern alternative with zero-cost wrappers (no unnecessary allocations).

```rust
use owo_colors::OwoColorize;

println!("{}", "Error".red().bold());
println!("{}", "RGB color".truecolor(255, 100, 50));
```

**Pros:** Extremely fast, supports RGB/TrueColor, color-blind friendly mode
**Best for:** High-performance CLI tools or libraries

### Handling Pipes (TTY Detection)

When users pipe your program's output to a file (e.g., `mytool > output.txt`), ANSI color codes can appear as garbage characters. Detect TTY and disable colors appropriately:

```rust
use std::io::IsTerminal;

if std::io::stdout().is_terminal() {
    println!("{}", "Colorful output".green());
} else {
    println!("Plain output");
}
```

## ANSI Escape Codes

Console escape codes are the hidden language of the terminal. They allow moving the cursor, changing colors, and even displaying images. Most sequences start with the Escape character (ASCII 27, hex `0x1B`).

### The Standards: ANSI, VT100, and Xterm

- **ANSI X3.64 / ISO/IEC 6429:** Primary standard for terminal control. Most sequences begin with CSI (Control Sequence Introducer): `ESC [` or `\033[`
- **DEC VT100:** The gold standard from the late 70s. Almost every modern terminal emulator (iTerm2, Windows Terminal, GNOME Terminal) mimics its behavior
- **Xterm:** Introduced extensions for mouse tracking, 256 colors, and window titles that are now universal

### Color Strategies

Terminal colors have evolved in "leaps" of bit-depth:

| Strategy | Sequence Example | Description |
| --- | --- | --- |
| **3-bit / 4-bit** | `\033[31m` | The original 8 colors (plus "bright" variants). Portable but limited. |
| **8-bit (256)** | `\033[38;5;124m` | Introduced by Xterm. Includes a 6x6x6 color cube and a grayscale ramp. |
| **24-bit (TrueColor)** | `\033[38;2;R;G;Bm` | Allows for 16.7 million colors. Supported by almost all modern emulators. |

**Pro Tip:** Check the `COLORTERM` environment variable. If set to `truecolor`, you can safely use 24-bit RGB.

### Common ANSI Sequences

```text
\033[0m      Reset all attributes
\033[1m      Bold
\033[2m      Dim
\033[3m      Italic
\033[4m      Underline
\033[7m      Reverse (swap fg/bg)
\033[31m     Red foreground
\033[42m     Green background
\033[2J      Clear screen
\033[H       Move cursor to home (0,0)
\033[K       Clear line from cursor to end
```

## Querying the Terminal

You don't just "talk" to the terminal; you can "listen" to it. These are Device Status Reports (DSR).

### Common Queries

- **Cursor Position:** Send `\033[6n` to ask where the cursor is. Terminal responds via stdin: `\033[R;C;R` (R=Row, C=Column)
- **Screen Dimensions:** Usually handled via `ioctl` system call (TIOCGWINSZ), but can query text area size in pixels using `\033[14t`
- **Theme Detection:** Query background color using OSC 11: `\033]11;?\007`. Vital for "Dark Mode" vs "Light Mode" detection

## Hyperlinks (OSC 8)

Create clickable text in the terminal that doesn't display the full URL.

**Format:** `\033]8;;URL\033\Text\033]8;;\033\`

Example:
```rust
println!("\x1b]8;;https://example.com\x1b\\Click here\x1b]8;;\x1b\\");
```

The user sees "Click here," but clicking opens "https://example.com" in their browser. Supported in VS Code, iTerm2, and Windows Terminal.

## Image Presentation

Displaying images in a grid of characters is hacky but impressive. Three main methods:

1. **Sixel:** Legacy DEC standard encoding images as 6-pixel high strips. Supported by MinTTY and Xterm.
2. **iTerm2 Protocol:** Encodes PNG/JPG as Base64 and sends via OSC sequence. Highest quality but mostly macOS.
3. **Kitty Graphics Protocol:** Most modern and robust. Uses shared memory or temporary files to pass pixel data to the terminal.

## Raw Mode vs. Cooked Mode

By default, your terminal operates in **Canonical (Cooked) Mode**.

### Cooked Mode (Default)

- OS buffers input until `Enter` is pressed
- OS handles `backspace` and `Ctrl+C`
- Program receives complete lines

### Raw Mode

- Program receives every keystroke immediately as it happens
- How editors like Vim or Nano work
- Program is responsible for everything (backspace sends `\x7f` byte; program must manually erase)

### Enabling Raw Mode (Rust with crossterm)

```rust
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

enable_raw_mode()?;
// Read keystroke by keystroke...
disable_raw_mode()?;
```

### The Trade-off

Raw mode gives you complete control but requires handling all terminal interactions manually. Use libraries like `crossterm` or `termion` to manage complexity.

## Related

- [Libraries](./libraries.md) - CLI libraries and frameworks
- [Design Patterns](./design-patterns.md) - CLI structure and conventions
