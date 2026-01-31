//! Performance benchmarks for the matching engine.
//!
//! Measures the performance of MatcherEngine::find_match() under various scenarios:
//! - Different library sizes (10, 100, 1000, 5000 combos)
//! - Strict vs loose matching modes
//! - Case-sensitive vs case-insensitive matching
//!
//! Run with: `cargo bench --bench matching_bench`
//! HTML reports will be generated in `target/criterion/`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use muttontext_lib::managers::matching::MatcherEngine;
use muttontext_lib::models::{Combo, ComboBuilder, MatchingMode};
use std::time::Duration;

/// Generates a vector of random-length keywords for realistic testing.
fn generate_keywords(count: usize) -> Vec<String> {
    let mut keywords = Vec::with_capacity(count);
    let prefixes = ["kw", "exp", "snip", "text", "auto", "sig", "addr", "tel", "email", "url"];
    let suffixes = ["", "x", "2", "v2", "new", "old", "temp", "draft", "final"];

    for i in 0..count {
        let prefix = prefixes[i % prefixes.len()];
        let suffix = suffixes[(i / prefixes.len()) % suffixes.len()];
        let keyword = if suffix.is_empty() {
            format!("{}{:04}", prefix, i)
        } else {
            format!("{}{:04}{}", prefix, i, suffix)
        };
        keywords.push(keyword);
    }

    keywords
}

/// Creates test combos with the specified matching mode and case sensitivity.
fn create_combos(
    count: usize,
    mode: MatchingMode,
    case_sensitive: bool,
) -> Vec<Combo> {
    let keywords = generate_keywords(count);
    let mut combos = Vec::with_capacity(count);

    for (i, keyword) in keywords.iter().enumerate() {
        let snippet = format!("Snippet content for combo {}", i);
        let combo = ComboBuilder::new()
            .keyword(keyword)
            .snippet(&snippet)
            .matching_mode(mode)
            .case_sensitive(case_sensitive)
            .build()
            .expect("Failed to build combo");
        combos.push(combo);
    }

    combos
}

/// Benchmarks find_match() with varying library sizes.
fn bench_library_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("library_size");
    group.measurement_time(Duration::from_secs(10));

    for size in [10, 100, 1000, 5000].iter() {
        // Create combos (mix of strict and loose)
        let strict_count = size / 2;
        let loose_count = size - strict_count;

        let mut combos = create_combos(strict_count, MatchingMode::Strict, false);
        combos.extend(create_combos(loose_count, MatchingMode::Loose, false));

        // Load into engine
        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        // Test buffer that matches the last strict combo
        let last_strict_keyword = format!("kw{:04}", strict_count - 1);
        let buffer = format!("hello {}", last_strict_keyword);

        group.bench_with_input(
            BenchmarkId::new("match_found", size),
            size,
            |b, _| {
                b.iter(|| {
                    let result = engine.find_match(black_box(&buffer), None);
                    black_box(result);
                });
            },
        );

        // Test buffer that doesn't match anything
        let no_match_buffer = "hello world xyz123 nomatch";
        group.bench_with_input(
            BenchmarkId::new("no_match", size),
            size,
            |b, _| {
                b.iter(|| {
                    let result = engine.find_match(black_box(no_match_buffer), None);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks strict vs loose matching modes separately.
fn bench_matching_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching_mode");
    group.measurement_time(Duration::from_secs(8));

    let size = 1000;

    // Strict mode benchmark
    {
        let combos = create_combos(size, MatchingMode::Strict, false);
        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        // Buffer with word boundary (should match)
        let match_buffer = "hello kw0500";
        group.bench_function("strict_match_with_boundary", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(match_buffer), None);
                black_box(result);
            });
        });

        // Buffer without word boundary (should NOT match in strict mode)
        let no_boundary_buffer = "hellokw0500";
        group.bench_function("strict_no_match_without_boundary", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(no_boundary_buffer), None);
                black_box(result);
            });
        });
    }

    // Loose mode benchmark
    {
        let combos = create_combos(size, MatchingMode::Loose, false);
        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        // Buffer with word boundary (should match)
        let match_buffer = "hello kw0500";
        group.bench_function("loose_match_with_boundary", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(match_buffer), None);
                black_box(result);
            });
        });

        // Buffer without word boundary (should STILL match in loose mode)
        let no_boundary_buffer = "hellokw0500";
        group.bench_function("loose_match_without_boundary", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(no_boundary_buffer), None);
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmarks case-sensitive vs case-insensitive matching.
fn bench_case_sensitivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("case_sensitivity");
    group.measurement_time(Duration::from_secs(8));

    let size = 1000;

    // Case-insensitive benchmark
    {
        let combos = create_combos(size, MatchingMode::Strict, false);
        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        let lowercase_buffer = "hello kw0500";
        group.bench_function("case_insensitive_lowercase", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(lowercase_buffer), None);
                black_box(result);
            });
        });

        let uppercase_buffer = "hello KW0500";
        group.bench_function("case_insensitive_uppercase", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(uppercase_buffer), None);
                black_box(result);
            });
        });

        let mixedcase_buffer = "hello Kw0500";
        group.bench_function("case_insensitive_mixedcase", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(mixedcase_buffer), None);
                black_box(result);
            });
        });
    }

    // Case-sensitive benchmark
    {
        let combos = create_combos(size, MatchingMode::Strict, true);
        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        let lowercase_buffer = "hello kw0500";
        group.bench_function("case_sensitive_match", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(lowercase_buffer), None);
                black_box(result);
            });
        });

        let uppercase_buffer = "hello KW0500";
        group.bench_function("case_sensitive_no_match", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(uppercase_buffer), None);
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmarks varying keyword lengths.
fn bench_keyword_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyword_length");
    group.measurement_time(Duration::from_secs(8));

    let size = 1000;

    // Short keywords (3-5 chars)
    {
        let keywords: Vec<String> = (0..size).map(|i| format!("k{}", i)).collect();
        let combos: Vec<Combo> = keywords
            .iter()
            .map(|kw| {
                ComboBuilder::new()
                    .keyword(kw)
                    .snippet("snippet")
                    .matching_mode(MatchingMode::Strict)
                    .build()
                    .expect("Failed to build combo")
            })
            .collect();

        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        let buffer = "hello k500";
        group.bench_function("short_keyword_3_5_chars", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(buffer), None);
                black_box(result);
            });
        });
    }

    // Medium keywords (8-12 chars)
    {
        let keywords: Vec<String> = (0..size).map(|i| format!("keyword{:04}", i)).collect();
        let combos: Vec<Combo> = keywords
            .iter()
            .map(|kw| {
                ComboBuilder::new()
                    .keyword(kw)
                    .snippet("snippet")
                    .matching_mode(MatchingMode::Strict)
                    .build()
                    .expect("Failed to build combo")
            })
            .collect();

        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        let buffer = "hello keyword0500";
        group.bench_function("medium_keyword_8_12_chars", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(buffer), None);
                black_box(result);
            });
        });
    }

    // Long keywords (15+ chars)
    {
        let keywords: Vec<String> = (0..size)
            .map(|i| format!("verylongkeyword{:04}", i))
            .collect();
        let combos: Vec<Combo> = keywords
            .iter()
            .map(|kw| {
                ComboBuilder::new()
                    .keyword(kw)
                    .snippet("snippet")
                    .matching_mode(MatchingMode::Strict)
                    .build()
                    .expect("Failed to build combo")
            })
            .collect();

        let mut engine = MatcherEngine::new();
        engine.load_combos(&combos);

        let buffer = "hello verylongkeyword0500";
        group.bench_function("long_keyword_15plus_chars", |b| {
            b.iter(|| {
                let result = engine.find_match(black_box(buffer), None);
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmarks engine loading performance.
fn bench_engine_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_loading");
    group.measurement_time(Duration::from_secs(8));

    for size in [100, 1000, 5000].iter() {
        let combos = create_combos(*size, MatchingMode::Strict, false);

        group.bench_with_input(
            BenchmarkId::new("load_combos", size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut engine = MatcherEngine::new();
                    engine.load_combos(black_box(&combos));
                    black_box(engine);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_library_sizes,
    bench_matching_modes,
    bench_case_sensitivity,
    bench_keyword_lengths,
    bench_engine_loading,
);
criterion_main!(benches);
