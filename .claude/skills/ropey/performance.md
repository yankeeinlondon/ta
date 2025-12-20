# Performance Tuning

Understanding Ropey's performance characteristics helps you optimize your text processing applications.

## Memory Efficiency

### Loading Overhead

- Fresh file load: ~10% overhead (100MB file uses ~110MB RAM)
- Built from random inserts: up to ~60% overhead

### Clone Efficiency

Ropey uses copy-on-write semantics:

```rust
let rope1 = Rope::from_str("Hello, world!");
let rope2 = rope1.clone();  // Only 8 bytes initially

// Memory grows incrementally as clones diverge through edits
rope2.insert(0, "Modified: ");
```

## Speed Characteristics

### Insertion Performance

On modern CPUs (mobile i7):
- **Incoherent insertions** (random locations): 1.8M ops/sec on ~100MB
- **Coherent insertions** (same location): 3.3M ops/sec on ~100MB

### Edit Latency

Single-digit microseconds even on GB-sized documents, ensuring responsive UIs.

## SIMD Acceleration

Ropey includes SIMD optimizations for bulk text processing:
- Character counting
- Line break detection
- UTF-8 validation

These activate automatically on supported hardware (x86_64, ARM with NEON).

## Patterns

### Batch Operations

When applying multiple changes, consider ordering:

```rust
// Apply changes in reverse order to preserve indices
changes.sort_by(|a, b| b.0.cmp(&a.0));
for (from, to, text) in changes {
    rope.remove(from..to);
    if let Some(text) = text {
        rope.insert(from, &text);
    }
}
```

**When to use:** Applying multiple edits from a diff or transaction.

### Use RopeBuilder for Construction

```rust
// Faster than repeated inserts for building large texts
use ropey::RopeBuilder;

let mut builder = RopeBuilder::new();
for chunk in source_chunks {
    builder.append(chunk);
}
let rope = builder.finish();
```

**When to use:** Building ropes from multiple pieces (file loading, concatenation).

## Size Thresholds

| Text Size | Recommendation |
|-----------|----------------|
| < 2KB | Consider `String` instead |
| 2KB - 100MB | Sweet spot for Ropey |
| 100MB - RAM | Works well, watch memory |
| > RAM | Not supported (in-memory only) |

## Related

- [Low-Level Chunk Access](./chunk-access.md)
