//! Performance benchmarks for Maguffin
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use url::Url;

/// Benchmark JSON serialization of PR-like structures
fn bench_json_serialization(c: &mut Criterion) {
    // Simulate a PR data structure
    let pr_data = serde_json::json!({
        "number": 42,
        "title": "Add new feature",
        "body": "This is a pull request description with some content.",
        "state": "open",
        "is_draft": false,
        "head_ref_name": "feature/new-feature",
        "base_ref_name": "main",
        "author": {
            "login": "testuser",
            "avatar_url": "https://example.com/avatar.png"
        },
        "labels": [
            {"name": "enhancement", "color": "84b6eb"},
            {"name": "ready-for-review", "color": "0e8a16"}
        ],
        "review_decision": "APPROVED",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-02T00:00:00Z",
        "additions": 100,
        "deletions": 50,
        "changed_files": 5,
        "commits_count": 3
    });

    c.bench_function("json_serialize_pr", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&pr_data).unwrap());
        })
    });

    c.bench_function("json_deserialize_pr", |b| {
        let json_str = serde_json::to_string(&pr_data).unwrap();
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(&json_str).unwrap());
        })
    });
}

/// Benchmark stack metadata operations
fn bench_stack_operations(c: &mut Criterion) {
    // Simulate stack metadata
    let mut stack_data: HashMap<String, Vec<String>> = HashMap::new();

    // Build a stack with 10 branches
    for i in 0..10 {
        stack_data.insert(
            format!("branch_{}", i),
            vec![
                format!("commit_{}", i * 10),
                format!("commit_{}", i * 10 + 1),
            ],
        );
    }

    c.bench_function("stack_lookup", |b| {
        b.iter(|| {
            black_box(stack_data.get("branch_5"));
        })
    });

    c.bench_function("stack_iteration", |b| {
        b.iter(|| {
            let count: usize = black_box(stack_data.iter().map(|(_, commits)| commits.len()).sum());
            count
        })
    });
}

/// Benchmark URL parsing operations using the url crate
fn bench_url_parsing(c: &mut Criterion) {
    let urls = [
        "https://github.com/owner/repo.git",
        "git@github.com:owner/repo.git",
        "ssh://git@github.com/owner/repo.git",
        "https://github.example.com/owner/repo.git",
    ];

    c.bench_function("parse_github_url", |b| {
        b.iter(|| {
            for url_str in &urls {
                // Use the url crate for real URL parsing
                if let Ok(parsed) = Url::parse(url_str) {
                    black_box(parsed.host_str());
                    black_box(parsed.path_segments().map(|s| s.collect::<Vec<_>>()));
                }
            }
        })
    });
}

/// Benchmark cache-like operations
fn bench_cache_operations(c: &mut Criterion) {
    let mut cache: HashMap<String, String> = HashMap::new();

    // Pre-populate cache
    for i in 0..100 {
        cache.insert(format!("key_{}", i), format!("value_{}", i));
    }

    c.bench_function("cache_get", |b| {
        b.iter(|| {
            black_box(cache.get("key_50"));
        })
    });

    c.bench_function("cache_insert", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            let mut cache_new = HashMap::new();
            cache_new.insert(format!("new_key_{}", counter), "new_value".to_string());
            counter = counter.wrapping_add(1);
            black_box(&cache_new);
        })
    });
}

criterion_group!(
    benches,
    bench_json_serialization,
    bench_stack_operations,
    bench_url_parsing,
    bench_cache_operations,
);
criterion_main!(benches);
