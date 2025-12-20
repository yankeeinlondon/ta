# Low-Level Chunk Access

Ropey provides APIs for efficiently accessing its internal text chunk representation, enabling custom functionality with minimal overhead.

## Key Concepts

Ropey stores text in approximately 1KB chunks arranged in a balanced tree. For operations not directly supported by Ropey's API, you can work directly with these chunks.

## Chunk Fetching Methods

### By Byte Index

```rust
let (chunk, chunk_byte_start, chunk_char_start, chunk_line_start) =
    rope.chunk_at_byte(byte_idx);
```

Returns the chunk containing the byte index along with the chunk's starting positions in bytes, chars, and lines.

### By Char Index

```rust
let (chunk, chunk_byte_start, chunk_char_start, chunk_line_start) =
    rope.chunk_at_char(char_idx);
```

### By Line Index

```rust
let (chunk, chunk_byte_start, chunk_char_start, chunk_line_start) =
    rope.chunk_at_line_break(line_idx);
```

## Chunks Iterator

For bulk processing, iterate over all chunks:

```rust
for chunk in rope.chunks() {
    // chunk is a &str
    process_chunk(chunk);
}
```

## Patterns

### Custom Byte-to-Char Conversion

```rust
use ropey::{Rope, str_utils::byte_to_char_idx};

fn byte_to_char(rope: &Rope, byte_idx: usize) -> usize {
    let (chunk, b, c, _) = rope.chunk_at_byte(byte_idx);
    c + byte_to_char_idx(chunk, byte_idx - b)
}
```

**When to use:** When you need byte-to-char conversion and want performance equivalent to Ropey's built-in implementation.

### UTF-16 Index Conversion

```rust
// Convert between char indices and UTF-16 code unit indices
// Useful for interop with external APIs using UTF-16 (e.g., LSP)
use ropey::str_utils::{byte_to_char_idx, char_to_byte_idx};
```

**When to use:** Interfacing with UTF-16 based protocols or systems.

## Related

- [Performance Tuning](./performance.md)
