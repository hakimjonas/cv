/// Performance tests for the Blog API
///
/// This file contains performance tests for the Blog API using criterion,
/// a statistics-driven benchmarking library for Rust.
///
/// To run these tests, you need to have the blog API server running on localhost:3000.
///
/// ```bash
/// # Start the blog API server
/// cargo run --bin blog_api_server
///
/// # Run the performance tests
/// cargo bench --bench blog_api_performance
/// ```
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

/// Benchmark creating a blog post
fn bench_create_post(c: &mut Criterion) {
    let client = Client::new();
    let mut group = c.benchmark_group("blog_api_create_post");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);
    
    for i in 0..5 {
        let post_number = i + 1;
        group.bench_with_input(BenchmarkId::new("create_post", post_number), &post_number, |b, &post_number| {
            b.iter(|| {
                let timestamp = chrono::Utc::now().timestamp();
                let post = json!({
                    "title": format!("Performance Test Post {}", post_number),
                    "slug": format!("perf-test-post-{}-{}", post_number, timestamp),
                    "date": "2025-07-23",
                    "author": "Performance Test",
                    "excerpt": "This is a test post created by a performance test",
                    "content": "# Performance Test Post\n\nThis post was created by an automated performance test.",
                    "published": true,
                    "featured": false,
                    "tags": [
                        {
                            "name": "Test",
                            "slug": "test"
                        }
                    ]
                });
                
                let response = client.post("http://localhost:3000/api/blog")
                    .json(&post)
                    .send()
                    .expect("Failed to send POST request");
                
                black_box(response.status().is_success());
            });
        });
    }
    
    group.finish();
}

/// Benchmark getting all blog posts
fn bench_get_all_posts(c: &mut Criterion) {
    let client = Client::new();
    let mut group = c.benchmark_group("blog_api_get_all_posts");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    group.bench_function("get_all_posts", |b| {
        b.iter(|| {
            let response = client.get("http://localhost:3000/api/blog")
                .send()
                .expect("Failed to send GET request");
            
            black_box(response.status().is_success());
        });
    });
    
    group.finish();
}

/// Benchmark getting a specific blog post
fn bench_get_post_by_slug(c: &mut Criterion) {
    let client = Client::new();
    
    // Create a test post to benchmark against
    let timestamp = chrono::Utc::now().timestamp();
    let post = json!({
        "title": "Performance Test Post for Get",
        "slug": format!("perf-test-post-get-{}", timestamp),
        "date": "2025-07-23",
        "author": "Performance Test",
        "excerpt": "This is a test post created for get performance testing",
        "content": "# Performance Test Post\n\nThis post was created for get performance testing.",
        "published": true,
        "featured": false,
        "tags": [
            {
                "name": "Test",
                "slug": "test"
            }
        ]
    });
    
    let response = client.post("http://localhost:3000/api/blog")
        .json(&post)
        .send()
        .expect("Failed to create test post");
    
    assert!(response.status().is_success(), "Failed to create test post");
    
    let slug = format!("perf-test-post-get-{}", timestamp);
    
    let mut group = c.benchmark_group("blog_api_get_post_by_slug");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    group.bench_with_input(BenchmarkId::new("get_post_by_slug", &slug), &slug, |b, slug| {
        b.iter(|| {
            let url = format!("http://localhost:3000/api/blog/{}", slug);
            let response = client.get(&url)
                .send()
                .expect("Failed to send GET request");
            
            black_box(response.status().is_success());
        });
    });
    
    group.finish();
    
    // Clean up the test post
    let url = format!("http://localhost:3000/api/blog/{}", slug);
    let response = client.delete(&url)
        .send()
        .expect("Failed to delete test post");
    
    assert!(response.status().is_success(), "Failed to delete test post");
}

/// Benchmark updating a blog post
fn bench_update_post(c: &mut Criterion) {
    let client = Client::new();
    
    // Create a test post to benchmark against
    let timestamp = chrono::Utc::now().timestamp();
    let slug = format!("perf-test-post-update-{}", timestamp);
    let post = json!({
        "title": "Performance Test Post for Update",
        "slug": &slug,
        "date": "2025-07-23",
        "author": "Performance Test",
        "excerpt": "This is a test post created for update performance testing",
        "content": "# Performance Test Post\n\nThis post was created for update performance testing.",
        "published": true,
        "featured": false,
        "tags": [
            {
                "name": "Test",
                "slug": "test"
            }
        ]
    });
    
    let response = client.post("http://localhost:3000/api/blog")
        .json(&post)
        .send()
        .expect("Failed to create test post");
    
    assert!(response.status().is_success(), "Failed to create test post");
    
    let mut group = c.benchmark_group("blog_api_update_post");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);
    
    group.bench_with_input(BenchmarkId::new("update_post", &slug), &slug, |b, slug| {
        b.iter(|| {
            let update_timestamp = chrono::Utc::now().timestamp();
            let updated_post = json!({
                "title": format!("Updated Performance Test Post {}", update_timestamp),
                "slug": slug,
                "date": "2025-07-23",
                "author": "Performance Test",
                "excerpt": "This is an updated test post",
                "content": format!("# Updated Performance Test Post\n\nThis post was updated at {}.", update_timestamp),
                "published": true,
                "featured": false,
                "tags": [
                    {
                        "name": "Test",
                        "slug": "test"
                    },
                    {
                        "name": "Updated",
                        "slug": "updated"
                    }
                ]
            });
            
            let url = format!("http://localhost:3000/api/blog/{}", slug);
            let response = client.put(&url)
                .json(&updated_post)
                .send()
                .expect("Failed to send PUT request");
            
            black_box(response.status().is_success());
        });
    });
    
    group.finish();
    
    // Clean up the test post
    let url = format!("http://localhost:3000/api/blog/{}", slug);
    let response = client.delete(&url)
        .send()
        .expect("Failed to delete test post");
    
    assert!(response.status().is_success(), "Failed to delete test post");
}

/// Benchmark deleting a blog post
fn bench_delete_post(c: &mut Criterion) {
    let client = Client::new();
    let mut group = c.benchmark_group("blog_api_delete_post");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);
    
    for i in 0..5 {
        let post_number = i + 1;
        group.bench_with_input(BenchmarkId::new("delete_post", post_number), &post_number, |b, &post_number| {
            b.iter_with_setup(
                || {
                    // Setup: Create a post to delete
                    let timestamp = chrono::Utc::now().timestamp();
                    let slug = format!("perf-test-post-delete-{}-{}", post_number, timestamp);
                    let post = json!({
                        "title": format!("Performance Test Post for Delete {}", post_number),
                        "slug": &slug,
                        "date": "2025-07-23",
                        "author": "Performance Test",
                        "excerpt": "This is a test post created for delete performance testing",
                        "content": "# Performance Test Post\n\nThis post was created for delete performance testing.",
                        "published": true,
                        "featured": false,
                        "tags": [
                            {
                                "name": "Test",
                                "slug": "test"
                            }
                        ]
                    });
                    
                    let response = client.post("http://localhost:3000/api/blog")
                        .json(&post)
                        .send()
                        .expect("Failed to create test post");
                    
                    assert!(response.status().is_success(), "Failed to create test post");
                    
                    slug
                },
                |slug| {
                    // Benchmark: Delete the post
                    let url = format!("http://localhost:3000/api/blog/{}", slug);
                    let response = client.delete(&url)
                        .send()
                        .expect("Failed to send DELETE request");
                    
                    black_box(response.status().is_success());
                }
            );
        });
    }
    
    group.finish();
}

/// Benchmark getting published blog posts
fn bench_get_published_posts(c: &mut Criterion) {
    let client = Client::new();
    let mut group = c.benchmark_group("blog_api_get_published_posts");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    group.bench_function("get_published_posts", |b| {
        b.iter(|| {
            let response = client.get("http://localhost:3000/api/blog/published")
                .send()
                .expect("Failed to send GET request");
            
            black_box(response.status().is_success());
        });
    });
    
    group.finish();
}

/// Benchmark getting posts by tag
fn bench_get_posts_by_tag(c: &mut Criterion) {
    let client = Client::new();
    let mut group = c.benchmark_group("blog_api_get_posts_by_tag");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    
    group.bench_function("get_posts_by_tag", |b| {
        b.iter(|| {
            let response = client.get("http://localhost:3000/api/blog/tag/test")
                .send()
                .expect("Failed to send GET request");
            
            black_box(response.status().is_success());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_create_post,
    bench_get_all_posts,
    bench_get_post_by_slug,
    bench_update_post,
    bench_delete_post,
    bench_get_published_posts,
    bench_get_posts_by_tag
);
criterion_main!(benches);