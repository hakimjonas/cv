/// Property-based tests for blog data transformations
///
/// This module contains property-based tests for the blog data transformations,
/// particularly the conversion functions between the blog_data and repository
/// BlogPost types.
extern crate cv;

use anyhow::Result;
use cv::blog_converters;
use cv::blog_data;
use cv::db::repository;
use cv::logging;
use im::{HashMap, Vector};
use proptest::prelude::*;
use tracing::{error, info};

fn main() -> Result<()> {
    // Initialize logging
    let config = logging::LoggingConfig {
        app_name: "blog_property_test".to_string(),
        level: tracing::Level::INFO,
        log_spans: true,
        ..Default::default()
    };
    let _guard = logging::init_logging(config);

    info!("Running property-based tests for blog data transformations");

    // Run the tests
    if let Err(e) = run_tests() {
        error!("Tests failed: {}", e);
        return Err(e.into());
    }

    info!("All tests passed!");
    Ok(())
}

fn run_tests() -> Result<()> {
    // Run the tests manually
    info!("Testing blog post roundtrip conversion...");
    test_blog_post_roundtrip_manual()?;

    info!("Testing blog posts vector conversion...");
    test_blog_posts_vector_conversion_manual()?;

    info!("Testing tags conversion...");
    test_tags_conversion_manual()?;

    info!("Testing metadata conversion...");
    test_metadata_conversion_manual()?;

    Ok(())
}

// Manual versions of the tests for running in main
fn test_blog_post_roundtrip_manual() -> Result<()> {
    // Create a simple blog post for testing
    let post = blog_data::BlogPost::new()
        .with_updated_title("Test Post")
        .with_updated_content("Test content")
        .with_updated_excerpt("Test excerpt")
        .with_updated_author("Test Author");

    // Convert to repository format
    let repo_post = blog_converters::data_to_repo(&post);

    // Convert back to data format
    let roundtrip_post = blog_converters::repo_to_data(&repo_post);

    // Check that the roundtrip post is equal to the original
    assert_eq!(post.id, roundtrip_post.id);
    assert_eq!(post.title, roundtrip_post.title);
    assert_eq!(post.slug, roundtrip_post.slug);
    assert_eq!(post.date, roundtrip_post.date);
    assert_eq!(post.author, roundtrip_post.author);
    assert_eq!(post.excerpt, roundtrip_post.excerpt);
    assert_eq!(post.content, roundtrip_post.content);
    assert_eq!(post.published, roundtrip_post.published);
    assert_eq!(post.featured, roundtrip_post.featured);
    assert_eq!(post.image, roundtrip_post.image);

    info!("Blog post roundtrip conversion test passed");
    Ok(())
}

fn test_blog_posts_vector_conversion_manual() -> Result<()> {
    // Create a vector of blog posts for testing
    let posts = Vector::from(vec![
        blog_data::BlogPost::new().with_updated_title("Post 1"),
        blog_data::BlogPost::new().with_updated_title("Post 2"),
        blog_data::BlogPost::new().with_updated_title("Post 3"),
    ]);

    // Convert to repository format
    let repo_posts = blog_converters::convert_posts_to_repo(&posts);

    // Convert back to data format
    let roundtrip_posts = blog_converters::convert_posts_to_data(&repo_posts);

    // Check that the number of posts is preserved
    assert_eq!(posts.len(), roundtrip_posts.len());

    // Check that each post is preserved
    for (i, (original_post, roundtrip_post)) in posts.iter().zip(roundtrip_posts.iter()).enumerate()
    {
        assert_eq!(
            original_post.id, roundtrip_post.id,
            "Post {} ID mismatch",
            i
        );
        assert_eq!(
            original_post.title, roundtrip_post.title,
            "Post {} title mismatch",
            i
        );
        assert_eq!(
            original_post.slug, roundtrip_post.slug,
            "Post {} slug mismatch",
            i
        );
    }

    info!("Blog posts vector conversion test passed");
    Ok(())
}

fn test_tags_conversion_manual() -> Result<()> {
    // Create a vector of tags for testing
    let tags = Vector::from(vec![
        blog_data::Tag {
            id: Some(1),
            name: "Tag 1".to_string(),
            slug: "tag-1".to_string(),
        },
        blog_data::Tag {
            id: Some(2),
            name: "Tag 2".to_string(),
            slug: "tag-2".to_string(),
        },
        blog_data::Tag {
            id: Some(3),
            name: "Tag 3".to_string(),
            slug: "tag-3".to_string(),
        },
    ]);

    // Convert to repository format
    let repo_tags = blog_converters::convert_tags_to_repo(&tags);

    // Convert back to data format
    let roundtrip_tags = blog_converters::convert_tags_to_data(&repo_tags);

    // Check that the number of tags is preserved
    assert_eq!(tags.len(), roundtrip_tags.len());

    // Check that each tag is preserved
    for (i, (original_tag, roundtrip_tag)) in tags.iter().zip(roundtrip_tags.iter()).enumerate() {
        assert_eq!(original_tag.id, roundtrip_tag.id, "Tag {} ID mismatch", i);
        assert_eq!(
            original_tag.name, roundtrip_tag.name,
            "Tag {} name mismatch",
            i
        );
        assert_eq!(
            original_tag.slug, roundtrip_tag.slug,
            "Tag {} slug mismatch",
            i
        );
    }

    info!("Tags conversion test passed");
    Ok(())
}

fn test_metadata_conversion_manual() -> Result<()> {
    // Create a HashMap of metadata for testing
    let mut metadata = HashMap::new();
    metadata = metadata.update("key1".to_string(), "value1".to_string());
    metadata = metadata.update("key2".to_string(), "value2".to_string());
    metadata = metadata.update("key3".to_string(), "value3".to_string());

    // Convert to repository format
    let repo_metadata = blog_converters::convert_metadata_to_repo(&metadata);

    // Convert back to data format
    let roundtrip_metadata = blog_converters::convert_metadata_to_data(&repo_metadata);

    // Check that the number of entries is preserved
    assert_eq!(metadata.len(), roundtrip_metadata.len());

    // Check that each key-value pair is preserved
    for (key, value) in metadata.iter() {
        assert_eq!(
            Some(value),
            roundtrip_metadata.get(key),
            "Metadata key {} value mismatch",
            key
        );
    }

    info!("Metadata conversion test passed");
    Ok(())
}

// Strategy for generating a Tag
fn tag_strategy() -> impl Strategy<Value = blog_data::Tag> {
    (prop::option::of(prop::num::i64::ANY), "\\PC*", "\\PC*")
        .prop_map(|(id, name, slug)| blog_data::Tag { id, name, slug })
}

// Strategy for generating a Vector of Tags
fn tags_strategy() -> impl Strategy<Value = Vector<blog_data::Tag>> {
    prop::collection::vec(tag_strategy(), 0..5).prop_map(|tags| tags.into_iter().collect())
}

// Strategy for generating a HashMap of metadata
fn metadata_strategy() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("\\PC*", "\\PC*", 0..5).prop_map(|map| {
        map.into_iter()
            .map(|(k, v)| (k, v))
            .collect::<HashMap<_, _>>()
    })
}

// Strategy for generating a BlogPost
fn blog_post_strategy() -> impl Strategy<Value = blog_data::BlogPost> {
    (
        prop::option::of(prop::num::i64::ANY),
        "\\PC*",
        "\\PC*",
        "\\PC*",
        "\\PC*",
        "\\PC*",
        "\\PC*",
        prop::bool::ANY,
        prop::bool::ANY,
        prop::option::of("\\PC*"),
        tags_strategy(),
        metadata_strategy(),
    )
        .prop_map(
            |(
                id,
                title,
                slug,
                date,
                author,
                excerpt,
                content,
                published,
                featured,
                image,
                tags,
                metadata,
            )| blog_data::BlogPost {
                id,
                title,
                slug,
                date,
                author,
                excerpt,
                content,
                published,
                featured,
                image,
                tags,
                metadata,
            },
        )
}

// Property: Converting a BlogPost to repository format and back should yield the original post
proptest! {
    #[test]
    fn test_blog_post_roundtrip(post in blog_post_strategy()) {
        // Convert to repository format
        let repo_post = blog_converters::data_to_repo(&post);

        // Convert back to data format
        let roundtrip_post = blog_converters::repo_to_data(&repo_post);

        // Check that the roundtrip post is equal to the original
        assert_eq!(post.id, roundtrip_post.id);
        assert_eq!(post.title, roundtrip_post.title);
        assert_eq!(post.slug, roundtrip_post.slug);
        assert_eq!(post.date, roundtrip_post.date);
        assert_eq!(post.author, roundtrip_post.author);
        assert_eq!(post.excerpt, roundtrip_post.excerpt);
        assert_eq!(post.content, roundtrip_post.content);
        assert_eq!(post.published, roundtrip_post.published);
        assert_eq!(post.featured, roundtrip_post.featured);
        assert_eq!(post.image, roundtrip_post.image);

        // Check that tags are preserved
        assert_eq!(post.tags.len(), roundtrip_post.tags.len());
        for (original_tag, roundtrip_tag) in post.tags.iter().zip(roundtrip_post.tags.iter()) {
            assert_eq!(original_tag.id, roundtrip_tag.id);
            assert_eq!(original_tag.name, roundtrip_tag.name);
            assert_eq!(original_tag.slug, roundtrip_tag.slug);
        }

        // Check that metadata is preserved
        assert_eq!(post.metadata.len(), roundtrip_post.metadata.len());
        for (key, value) in post.metadata.iter() {
            assert_eq!(Some(value), roundtrip_post.metadata.get(key));
        }
    }
}

// Property: Converting a Vector of BlogPosts should preserve the order and content
proptest! {
    #[test]
    fn test_blog_posts_vector_conversion(posts in prop::collection::vec(blog_post_strategy(), 0..10)) {
        let posts_vector: Vector<blog_data::BlogPost> = posts.into_iter().collect();

        // Convert to repository format
        let repo_posts = blog_converters::convert_posts_to_repo(&posts_vector);

        // Convert back to data format
        let roundtrip_posts = blog_converters::convert_posts_to_data(&repo_posts);

        // Check that the number of posts is preserved
        assert_eq!(posts_vector.len(), roundtrip_posts.len());

        // Check that each post is preserved
        for (original_post, roundtrip_post) in posts_vector.iter().zip(roundtrip_posts.iter()) {
            assert_eq!(original_post.id, roundtrip_post.id);
            assert_eq!(original_post.title, roundtrip_post.title);
            assert_eq!(original_post.slug, roundtrip_post.slug);
            // ... other fields can be checked similarly
        }
    }
}

// Property: Tags conversion should preserve all tag data
proptest! {
    #[test]
    fn test_tags_conversion(tags in tags_strategy()) {
        // Convert to repository format
        let repo_tags = blog_converters::convert_tags_to_repo(&tags);

        // Convert back to data format
        let roundtrip_tags = blog_converters::convert_tags_to_data(&repo_tags);

        // Check that the number of tags is preserved
        assert_eq!(tags.len(), roundtrip_tags.len());

        // Check that each tag is preserved
        for (original_tag, roundtrip_tag) in tags.iter().zip(roundtrip_tags.iter()) {
            assert_eq!(original_tag.id, roundtrip_tag.id);
            assert_eq!(original_tag.name, roundtrip_tag.name);
            assert_eq!(original_tag.slug, roundtrip_tag.slug);
        }
    }
}

// Property: Metadata conversion should preserve all key-value pairs
proptest! {
    #[test]
    fn test_metadata_conversion(metadata in metadata_strategy()) {
        // Convert to repository format
        let repo_metadata = blog_converters::convert_metadata_to_repo(&metadata);

        // Convert back to data format
        let roundtrip_metadata = blog_converters::convert_metadata_to_data(&repo_metadata);

        // Check that the number of entries is preserved
        assert_eq!(metadata.len(), roundtrip_metadata.len());

        // Check that each key-value pair is preserved
        for (key, value) in metadata.iter() {
            assert_eq!(Some(value), roundtrip_metadata.get(key));
        }
    }
}
