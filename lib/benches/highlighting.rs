use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ta_lib::highlighting::{highlight_code, HighlightOptions};

/// Generate TypeScript code with a specified number of lines
fn generate_typescript_code(lines: usize) -> String {
    let mut code = String::new();
    code.push_str("// TypeScript code for benchmarking\n\n");

    for i in 0..lines {
        match i % 5 {
            0 => code.push_str(&format!("const variable{} = {};\n", i, i)),
            1 => code.push_str(&format!(
                "function func{}(x: number): number {{ return x * {}; }}\n",
                i, i
            )),
            2 => code.push_str(&format!("type Type{} = {{ id: number; name: string }};\n", i)),
            3 => code.push_str(&format!(
                "interface Interface{} {{ method(): void; }}\n",
                i
            )),
            4 => code.push_str(&format!("class Class{} {{ private value = {}; }}\n", i, i)),
            _ => unreachable!(),
        }
    }

    code
}

fn benchmark_highlighting(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_highlighting");

    // Test different code sizes
    for size in [10, 50, 100, 500, 1000].iter() {
        let code = generate_typescript_code(*size);

        group.bench_with_input(
            BenchmarkId::new("typescript", size),
            &code,
            |b, code| {
                b.iter(|| {
                    highlight_code(
                        black_box(code),
                        black_box(HighlightOptions::new("typescript")),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_theme_selection(c: &mut Criterion) {
    let code = generate_typescript_code(100);

    c.bench_function("theme_default", |b| {
        b.iter(|| {
            highlight_code(
                black_box(&code),
                black_box(HighlightOptions::new("typescript")),
            )
        })
    });

    c.bench_function("theme_monokai", |b| {
        b.iter(|| {
            highlight_code(
                black_box(&code),
                black_box(
                    HighlightOptions::new("typescript").with_theme("Monokai Extended"),
                ),
            )
        })
    });
}

fn benchmark_different_languages(c: &mut Criterion) {
    let mut group = c.benchmark_group("languages");

    let typescript_code = "const x: number = 42; function add(a: number, b: number) { return a + b; }";
    let rust_code = "fn main() { let x: i32 = 42; println!(\"x = {}\", x); }";
    let python_code = "def add(a, b):\n    return a + b\n\nx = 42\nprint(x)";

    group.bench_function("typescript", |b| {
        b.iter(|| {
            highlight_code(
                black_box(typescript_code),
                black_box(HighlightOptions::new("typescript")),
            )
        })
    });

    group.bench_function("rust", |b| {
        b.iter(|| {
            highlight_code(black_box(rust_code), black_box(HighlightOptions::new("rust")))
        })
    });

    group.bench_function("python", |b| {
        b.iter(|| {
            highlight_code(
                black_box(python_code),
                black_box(HighlightOptions::new("python")),
            )
        })
    });

    group.finish();
}

fn benchmark_rendering(c: &mut Criterion) {
    let code = generate_typescript_code(100);
    let highlighted = highlight_code(&code, HighlightOptions::new("typescript"))
        .expect("Failed to highlight code");

    c.bench_function("render_console", |b| {
        b.iter(|| black_box(&highlighted).render_console())
    });

    c.bench_function("render_html", |b| {
        b.iter(|| black_box(&highlighted).render_html())
    });
}

criterion_group!(
    benches,
    benchmark_highlighting,
    benchmark_theme_selection,
    benchmark_different_languages,
    benchmark_rendering
);
criterion_main!(benches);
