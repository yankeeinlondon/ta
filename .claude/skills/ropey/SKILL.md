---
name: ropey
description: Expert knowledge for Ropey, a high-performance UTF-8 text rope library for Rust text editors, word processors, and large document manipulation with Unicode support, line tracking, and thread safety
hash: 1d01ad74ff5791e6
---

# Ropey

Ropey is a high-performance UTF-8 text rope data structure for Rust. It serves as a text buffer for applications requiring efficient manipulation of large texts such as text editors and word processors. Uses a piecewise tree structure for fast, memory-efficient editing operations.

## Core Principles

- **Unicode scalar values** are the atomic unit (Rust's `char`) - all operations maintain UTF-8 validity
- **Line-aware** - built-in recognition of line breaks including CRLF
- **Thread-safe** - clones share memory but can be sent to other threads safely
- **Copy-on-write** - clones cost only 8 bytes initially, growing as they diverge
- **Chunk-based** - text stored in ~1KB chunks in a balanced tree for O(log n) operations
- **SIMD accelerated** - bulk operations benefit from SIMD optimizations on supported hardware

## Quick Reference

### Basic Operations

```rust
use ropey::Rope;
use std::fs::File;
use std::io::{BufReader, BufWriter};

// Create from string
let rope = Rope::from_str("Hello, world!");

// Load from file
let rope = Rope::from_reader(BufReader::new(File::open("file.txt")?))?;

// Insert text at char index
rope.insert(5, " new text");

// Insert single char
rope.insert_char(5, 'X');

// Remove range (char indices)
rope.remove(start..end);

// Get line by index (0-indexed)
let line = rope.line(515);

// Convert line to char index
let start_idx = rope.line_to_char(515);
let end_idx = rope.line_to_char(516);

// Write to file
rope.write_to(BufWriter::new(File::create("output.txt")?))?;
```

### RopeSlice for Read-Only Views

```rust
// Get immutable slice
let slice = rope.slice(10..50);

// All read-only operations work on slices
println!("{}", slice.len_chars());
for line in slice.lines() {
    println!("{}", line);
}
```

### RopeBuilder for Incremental Construction

```rust
use ropey::RopeBuilder;

let mut builder = RopeBuilder::new();
builder.append("First chunk");
builder.append(" second chunk");
let rope = builder.finish();
```

## Topics

### Advanced Usage

- [Low-Level Chunk Access](./chunk-access.md) - Direct chunk manipulation for custom operations
- [Performance Tuning](./performance.md) - Memory and speed characteristics

## Common Patterns

### Apply Text Change

```rust
pub type Change = (usize, usize, Option<String>);

fn apply_change(rope: &mut Rope, change: Change) {
    let (from, to, text) = change;
    rope.remove(from..to);
    if let Some(text) = text {
        rope.insert(from, &text);
    }
}
```

### Byte to Char Conversion

```rust
use ropey::{Rope, str_utils::byte_to_char_idx};

fn byte_to_char(rope: &Rope, byte_idx: usize) -> usize {
    let (chunk, b, c, _) = rope.chunk_at_byte(byte_idx);
    c + byte_to_char_idx(chunk, byte_idx - b)
}
```

## Performance Characteristics

| Operation | Performance | Notes |
|-----------|-------------|-------|
| Incoherent insertions | 1.8M ops/sec | Random locations, ~100MB doc |
| Coherent insertions | 3.3M ops/sec | Same location, ~100MB doc |
| Edit latency | Microseconds | Even on GB-sized docs |
| Memory overhead | ~10% | Fresh file load |
| Clone cost | 8 bytes | Initial, grows on diverge |
| Worst-case overhead | ~60% | Many small random inserts |

## When to Use Ropey

**Good for:**
- Text editors and word processors
- Large document manipulation (MB to GB)
- Unicode-critical applications
- Multi-threaded text processing
- Performance-sensitive UIs

**Not ideal for:**
- Very small texts (< few KB) - chunk overhead wasteful
- Texts larger than RAM
- Simple use cases not needing Unicode/line tracking

## Alternatives Comparison

| Feature | Ropey | Crop | JumpRope |
|---------|-------|------|----------|
| Speed | Fast | Faster | ~3x Faster |
| Unicode | Full | Full | Limited |
| Line tracking | Yes | Yes | No |
| Thread safety | Yes | No | Yes |
| Memory | Good | Better | Best |

## Resources

- [Crates.io](https://crates.io/crates/ropey)
- [GitHub](https://github.com/cessen/ropey)
- [API Docs](https://docs.rs/ropey)
