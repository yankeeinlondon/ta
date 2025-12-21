---
name: cli
description: Comprehensive guide to building modern command-line interfaces, covering design patterns, terminal capabilities, and ecosystem tools for both Rust and TypeScript
created: 2025-12-20
last_updated: 2025-12-20T00:00:00Z
hash: e666af0507a9ba55
tags:
  - cli
  - terminal
  - rust
  - typescript
  - console
  - ansi
---

# Command Line Interface Development

Building great command-line interfaces requires understanding three interconnected domains: design patterns that make tools intuitive, the technical capabilities of modern terminals, and the ecosystem of libraries that accelerate development. This guide synthesizes these topics into a comprehensive reference for building professional CLI tools.

## Table of Contents

1. [CLI Design Patterns](#cli-design-patterns)
   - [Command Structure](#command-structure)
   - [POSIX Conventions](#posix-conventions)
   - [Feedback and Progress](#feedback-and-progress)
   - [Interactive Patterns](#interactive-patterns)
   - [Unix Philosophy](#unix-philosophy)
   - [Discoverability](#discoverability)
2. [Terminal Capabilities](#terminal-capabilities)
   - [Escape Code Standards](#escape-code-standards)
   - [Color Strategies](#color-strategies)
   - [Terminal Queries](#terminal-queries)
   - [Hyperlinks (OSC 8)](#hyperlinks-osc-8)
   - [Image Presentation](#image-presentation)
   - [Raw vs Cooked Mode](#raw-vs-cooked-mode)
3. [Ecosystem Tools](#ecosystem-tools)
   - [Rust CLI Libraries](#rust-cli-libraries)
   - [TypeScript CLI Libraries](#typescript-cli-libraries)
   - [Rust Color Crates](#rust-color-crates)
   - [Language Selection Guide](#language-selection-guide)

## CLI Design Patterns

Designing a Command Line Interface is a unique challenge. Unlike GUIs where users are guided by visual cues, a CLI relies on **predictability, discoverability, and composability**. Great CLIs follow the "Rule of Least Surprise."

### Command Structure

Most modern CLIs follow the **Command-Subcommand-Argument** pattern, popularized by Git. This creates a hierarchical, readable structure that scales well as features grow.

**Pattern:** `[binary] [group] [command] [arguments] --flags`

**Example:** `docker container run --detach alpine`

| Component | Role | Example |
| --- | --- | --- |
| **Binary** | The name of the tool | `git` |
| **Command/Group** | The object or action being targeted | `remote`, `checkout` |
| **Subcommand** | Refines the action | `add` |
| **Arguments** | Positional data (required) | `origin [URL]` |
| **Flags/Options** | Modifiers (optional) | `--force`, `-v` |

This hierarchical approach creates intuitive groupings. Users can explore capabilities by running `tool [group] --help` to see what actions are available within that group.

### POSIX Conventions

To make your tool feel "native" to terminal users, follow established conventions for flags:

**Short vs. Long Flags:**
- Short flags (`-f`) use a single dash and can be "bundled" (e.g., `tar -xvf`)
- Long flags (`--force`) use double dashes and are more readable in scripts

**The `--` Separator:** Use this to signal the end of flags and the beginning of positional arguments. This is vital if your argument starts with a dash (e.g., `rm -- -filename`).

**Booleans vs. Values:** Flags should either be toggles or explicitly take a value (e.g., `--output=json` or `--output json`).

### Feedback and Progress

Since there is no "screen" to refresh, how you communicate state is critical. Never leave the user with a blinking cursor during long operations.

**Progress Indicators:**
- **Spinners:** Use for tasks of unknown duration (e.g., `Fetching data...`)
- **Progress Bars:** Use for tasks with a known "total" (e.g., `Downloading [====>    ] 60%`)

**Log Level Pattern:** Allow users to control verbosity using `-v`, `-vv`, or `--quiet`.

**Color Pattern (ANSI):**
- **Success:** Green
- **Warnings:** Yellow/Orange
- **Errors:** Red
- **Rule:** Always detect if the output is a TTY (a real person watching). If output is being piped to a file (`my-tool > log.txt`), disable colors and animations automatically.

### Interactive Patterns

While many CLIs are non-interactive for automation, "Human-First" CLIs use interactive patterns:

- **Prompts:** Asking "Are you sure? [y/N]"
- **Selectable Lists:** Using arrow keys to pick an option (common in tools like `npm init` or `gh`)
- **Fuzzy Search:** Allowing users to type a few letters to filter a long list of resources

### Unix Philosophy

For a CLI to be powerful, it must play well with others:

**Standard Streams:** Use `stdout` for the actual data and `stderr` for logs/errors. This allows users to pipe data: `my-tool --json | jq .`

**Output Formatting:** Offer a `--json` or `--format` flag. This makes your CLI a "data source" for other scripts, moving it from a simple tool to a platform component.

**Idempotency:** Running the same command twice should (ideally) not cause an error or duplicate resources if the desired state is already met.

### Discoverability

- **The Global Help:** Every tool must support `-h` and `--help`
- **Auto-suggestions:** "Did you mean `status`?" when a user types `statas`
- **Man Pages:** For deep technical documentation accessible offline via `man [tool]`

## Terminal Capabilities

Console escape codes are the hidden language of the terminal. They allow developers to move the cursor, change colors, and even display images within a text-based interface. Most of these sequences start with the Escape character (ASCII 27, hex `0x1B`).

### Escape Code Standards

The world of terminal sequences is a mix of rigid ISO standards and "de facto" standards established by popular hardware:

**ANSI X3.64 / ISO/IEC 6429:** This is the primary standard for terminal control. Most sequences begin with the Control Sequence Introducer (CSI), which is `ESC [` (or `\033[`).

**DEC VT100:** The Gold Standard. In the late 70s, Digital Equipment Corporation's VT100 terminal became so popular that almost every modern terminal emulator (iTerm2, Windows Terminal, GNOME Terminal) still mimics its behavior.

**Xterm:** As terminal emulators moved to the desktop, Xterm introduced extensions for mouse tracking, 256 colors, and window titles that are now universal.

### Color Strategies

Terminal colors have evolved in "leaps" of bit-depth:

| Strategy | Sequence Example | Description |
| --- | --- | --- |
| **3-bit / 4-bit** | `\033[31m` | The original 8 colors (plus "bright" variants). Portable but limited. |
| **8-bit (256)** | `\033[38;5;124m` | Introduced by Xterm. Includes a 6x6x6 color cube and a grayscale ramp. |
| **24-bit (TrueColor)** | `\033[38;2;R;G;Bm` | Allows for 16.7 million colors. Supported by almost all modern emulators. |

**Pro Tip:** Always check the `COLORTERM` environment variable. If it's set to `truecolor`, you can safely use 24-bit RGB.

### Terminal Queries

You don't just "talk" to the terminal; you can "listen" to it. These are often called DSR (Device Status Reports):

**Cursor Position:** Sending `\033[6n` asks the terminal where the cursor is. The terminal responds via `stdin` with `\033[R;C;R` (where R=Row, C=Column).

**Screen Dimensions:** While usually handled via the `ioctl` system call in C (TIOCGWINSZ), you can query the terminal for text area size in pixels using `\033[14t`.

**Theme Detection:** You can query the background color using OSC 11 (`\033]11;?\007`). This is vital for "Dark Mode" vs "Light Mode" detection so your CLI remains readable.

### Hyperlinks (OSC 8)

One of the most useful modern additions is the OSC 8 standard. It allows you to create "clickable" text in the terminal that doesn't display the full, ugly URL.

**Format:** `\033]8;;URL\033\Text\033]8;;\033\`

If you print this, the user sees "Text," but clicking it opens the "URL" in their browser. This is now supported in VS Code, iTerm2, and Windows Terminal.

### Image Presentation

Displaying images in a grid of characters is a hacky but impressive feat. There are three main ways this is done:

1. **Sixel:** A legacy DEC standard that encodes images as a series of 6-pixel high strips. Supported by MinTTY and Xterm.
2. **iTerm2 Protocol:** Encodes a PNG/JPG as Base64 and sends it via an OSC sequence. This is the highest quality but mostly limited to macOS.
3. **Kitty Graphics Protocol:** The most modern and robust method. It uses shared memory or temporary files to pass pixel data to the terminal.

### Raw vs Cooked Mode

By default, your terminal operates in Canonical (Cooked) Mode:

**Cooked Mode:** The OS buffers your input. When you type, nothing is sent to the program until you hit `Enter`. The OS handles `backspace` and `Ctrl+C` for you.

**Raw Mode:** The program receives every keystroke immediately as it happens. This is how editors like Vim or Nano work.

**The Trade-off:** In Raw mode, the program is responsible for everything. If you type `backspace`, the program receives a `\x7f` byte; it must then manually move the cursor back and print a space to "erase" the character on the screen.

## Ecosystem Tools

Building a CLI involves several layers: parsing arguments, styling the output (colors/tables), and managing user interaction (prompts/progress bars). Both Rust and TypeScript have mature ecosystems that handle these tasks elegantly.

### Rust CLI Libraries

Rust is often considered the gold standard for modern CLI development because it produces small, fast, single-file binaries that don't require a runtime.

| Library | Category | Description |
| --- | --- | --- |
| `clap` | Arg Parsing | The industry standard. Uses a "derive" macro that lets you define CLI arguments using a simple Rust `struct`. |
| `ratatui` | TUI / UI | A powerful library for building full Terminal User Interfaces (rich dashboards and complex layouts). |
| `indicatif` | Feedback | The go-to library for beautiful progress bars and spinners. |
| `dialoguer` | Interaction | Provides interactive prompts for user input, password entry, and multi-select menus. |
| `crossterm` | Terminal Control | A cross-platform library to clear the screen, move the cursor, and handle keyboard events. |
| `anyhow` | Errors | Simplifies error handling in CLI apps so you don't have to define custom error types for every small task. |

### TypeScript CLI Libraries

TypeScript is excellent for CLIs that need to be built quickly or integrated into existing JavaScript-heavy workflows.

| Library | Category | Description |
| --- | --- | --- |
| `commander` | Arg Parsing | The most established library for defining commands, options, and help text. |
| `oclif` | Framework | A "batteries-included" framework from Salesforce designed for building large, professional-grade CLIs. |
| `inquirer` | Interaction | The classic library for interactive prompts (checklists, input, lists). |
| `chalk` | Styling | The standard for adding colors and styling to terminal text output. |
| `clack` | Interaction | A modern, high-polish alternative to Inquirer with a focus on "effortless" and beautiful UI. |
| `zod` | Validation | While not CLI-specific, frequently used with arg parsers to ensure input matches strict data types. |

### Rust Color Crates

Colorizing console output in Rust is a common task, and the ecosystem offers several excellent crates that range from low-level control to high-level convenience.

**1. Colored (Easiest to Use)**

If you want the quickest, most readable syntax, **`colored`** is the community favorite. It uses an extension trait to add coloring methods directly to strings.

- **Syntax:** `"Hello".red().bold()`
- **Pros:** Extremely intuitive; no complex setup
- **Best for:** Small tools, scripts, and quick debugging

**Example:**

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

To use `colored`, add `colored = "2"` to your `Cargo.toml`.

**2. Termcolor (Cross-Platform Standard)**

If you are building a professional CLI tool that needs to work perfectly on **Windows** (including older versions), **`termcolor`** is the industry standard. It's used by the Rust compiler itself and tools like `ripgrep`.

- **Mechanism:** Uses a "buffer" approach to ensure colors are handled correctly across different terminal emulators
- **Pros:** High performance, very reliable, handles "No Color" (NO_COLOR) environment variables well
- **Cons:** More boilerplate code compared to `colored`

**3. Owo-colors (Zero-Cost & Type-Safe)**

If you are worried about performance or want to avoid unnecessary allocations, **`owo-colors`** is a fantastic modern alternative.

- **Mechanism:** Uses "Zero-cost" wrappers, meaning it doesn't allocate new strings just to add color
- **Pros:** Extremely fast, supports RGB/TrueColor, and provides a "Color-Blind" friendly mode
- **Best for:** High-performance CLI tools or libraries

**Comparison Table:**

| Crate | Style | Learning Curve | Best Use Case |
| --- | --- | --- | --- |
| **`colored`** | `String` extensions | Very Easy | Prototyping & small scripts |
| **`termcolor`** | Buffer-based | Moderate | Professional, cross-platform tools |
| **`owo-colors`** | Wrapper-based | Easy | Performance-critical applications |
| **`nu-ansi-term`** | Struct-based | Easy | Complex styling and layout |

**Pro Tip: Handling Pipes**

When users pipe your program's output to another file (e.g., `my_app > output.txt`), ANSI color codes can look like "garbage" characters. Most of these crates (especially `termcolor` and `colored`) have features to detect if the output is a **TTY** (a real screen) and will automatically disable colors when being piped to a file.

### Language Selection Guide

**Choose Rust if:**
- You need high performance
- You want a single binary for distribution
- The tool will be used in CI/CD pipelines where startup time matters
- Cross-platform compatibility (including Windows) is critical

**Choose TypeScript if:**
- You are building tools for web developers
- You want to leverage the massive npm ecosystem
- You need to prototype an interactive tool very quickly
- Integration with existing JavaScript workflows is important

## Quick Reference

### Common ANSI Escape Sequences

```
\033[0m      Reset all attributes
\033[1m      Bold
\033[31m     Red foreground
\033[42m     Green background
\033[2J      Clear screen
\033[H       Move cursor to home
\033[K       Clear line from cursor
```

### Color Detection

```bash
# Check if terminal supports truecolor
if [ "$COLORTERM" = "truecolor" ]; then
    # Use 24-bit RGB
fi

# Check if output is a TTY (not piped)
if [ -t 1 ]; then
    # Use colors and animations
fi
```

### Essential CLI Flags

- `-h`, `--help`: Show help information
- `-v`, `--version`: Show version
- `-q`, `--quiet`: Suppress output
- `--verbose`: Increase output detail
- `--json`: Output in JSON format
- `--no-color`: Disable color output

## Resources

### Standards & Specifications
- [ANSI X3.64 / ISO/IEC 6429](https://www.iso.org/standard/12782.html) - Terminal control sequences
- [VT100 User Guide](https://vt100.net/docs/vt100-ug/) - DEC VT100 terminal documentation
- [Xterm Control Sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - Comprehensive reference

### Rust Resources
- [clap documentation](https://docs.rs/clap/) - Argument parsing
- [ratatui](https://ratatui.rs/) - Terminal UI framework
- [Rust CLI Working Group](https://rust-cli.github.io/) - Best practices and tools

### TypeScript Resources
- [commander.js](https://github.com/tj/commander.js) - Argument parsing
- [oclif](https://oclif.io/) - CLI framework
- [chalk](https://github.com/chalk/chalk) - Terminal styling

### Testing & Development
- [expect](https://github.com/ezio-melotti/python-expect) - Testing CLI interactions
- [tmux](https://github.com/tmux/tmux) - Terminal multiplexer for development
- [NO_COLOR standard](https://no-color.org/) - Environment variable standard
