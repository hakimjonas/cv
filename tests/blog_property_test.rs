use cv::db::repository::{BlogPost, BlogRepository, Tag};
use im::{HashMap, Vector};
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};
use proptest::test_runner::TestRunner;
use serde_json::{from_str, to_string};
use std::cell::RefCell;
use std::sync::Arc;
use tempfile::TempDir;

// Import the database module to create a test database
use cv::db;

// Generator for Tag
fn tag_strategy() -> BoxedStrategy<Tag> {
    // Generate a valid tag name (alphanumeric with spaces)
    let name_strategy = "[a-zA-Z0-9 ]{1,20}";

    name_strategy
        .prop_map(|name| {
            // Generate a slug from the name
            let slug = name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>();

            Tag {
                id: None,
                name,
                slug,
            }
        })
        .boxed()
}

// Generator for a vector of Tags
fn tags_strategy() -> BoxedStrategy<Vector<Tag>> {
    proptest::collection::vec(tag_strategy(), 0..5)
        .prop_map(|tags| tags.into_iter().collect())
        .boxed()
}

// Generator for metadata
fn metadata_strategy() -> BoxedStrategy<HashMap<String, String>> {
    let key_strategy = "[a-zA-Z0-9-]{1,10}";
    let value_strategy = "[a-zA-Z0-9 .,!?-]{0,30}";

    proptest::collection::hash_map(key_strategy, value_strategy, 0..5)
        .prop_map(|map| map.into_iter().collect())
        .boxed()
}

// Generator for BlogPost
fn blog_post_strategy() -> BoxedStrategy<BlogPost> {
    let title_strategy = "[a-zA-Z0-9 .,!?-]{3,50}";
    let content_strategy = "[a-zA-Z0-9 .,!?-]{10,200}";
    let excerpt_strategy = "[a-zA-Z0-9 .,!?-]{5,100}";
    let author_strategy = "[a-zA-Z ]{3,30}";
    let date_strategy = "\\d{4}-\\d{2}-\\d{2}";
    let image_strategy = prop_oneof![
        Just(None),
        "[a-zA-Z0-9/._-]{5,50}.jpg".prop_map(Some),
        "[a-zA-Z0-9/._-]{5,50}.png".prop_map(Some)
    ];

    (
        title_strategy,
        content_strategy,
        excerpt_strategy,
        author_strategy,
        date_strategy,
        image_strategy,
        tags_strategy(),
        metadata_strategy(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                title,
                content,
                excerpt,
                author,
                date,
                image,
                tags,
                metadata,
                published,
                featured,
            )| {
                // Generate a slug from the title
                let slug = title
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '-' })
                    .collect::<String>();

                BlogPost {
                    id: None,
                    title,
                    slug,
                    content,
                    excerpt,
                    author,
                    date,
                    image,
                    tags,
                    metadata,
                    published,
                    featured,
                }
            },
        )
        .boxed()
}

// Helper function to create a test database and repository
async fn setup_test_db() -> (TempDir, Arc<BlogRepository>) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create a database connection pool
    let pool = db::create_connection_pool(db_path.to_str().unwrap()).unwrap();

    // Run migrations to set up the schema
    db::run_migrations_async(&pool).await.unwrap();

    // Create a repository
    let repo = Arc::new(BlogRepository::new(Arc::new(pool)));

    (temp_dir, repo)
}

// Property: Serialization/deserialization roundtrip for Tag
proptest! {
    #[test]
    fn tag_serde_roundtrip(tag in tag_strategy()) {
        // Serialize to JSON
        let json = to_string(&tag).unwrap();

        // Deserialize back
        let deserialized_tag: Tag = from_str(&json).unwrap();

        // Check equality
        prop_assert_eq!(tag, deserialized_tag);
    }
}

// Property: Serialization/deserialization roundtrip for BlogPost
proptest! {
    #[test]
    fn blog_post_serde_roundtrip(post in blog_post_strategy()) {
        // Serialize to JSON
        let json = to_string(&post).unwrap();

        // Deserialize back
        let deserialized_post: BlogPost = from_str(&json).unwrap();

        // Check equality
        prop_assert_eq!(post.title, deserialized_post.title);
        prop_assert_eq!(post.slug, deserialized_post.slug);
        prop_assert_eq!(post.content, deserialized_post.content);
        prop_assert_eq!(post.excerpt, deserialized_post.excerpt);
        prop_assert_eq!(post.author, deserialized_post.author);
        prop_assert_eq!(post.date, deserialized_post.date);
        prop_assert_eq!(post.image, deserialized_post.image);
        prop_assert_eq!(post.tags, deserialized_post.tags);
        prop_assert_eq!(post.metadata, deserialized_post.metadata);
        prop_assert_eq!(post.published, deserialized_post.published);
        prop_assert_eq!(post.featured, deserialized_post.featured);
    }
}

// Property: Saving and retrieving a post preserves all data
#[tokio::test]
async fn save_and_retrieve_post_preserves_data() {
    let mut runner = TestRunner::default();

    // Generate a random blog post
    let post_cell = RefCell::new(None);
    runner
        .run(&blog_post_strategy(), |p| {
            post_cell.replace(Some(p.clone()));
            Ok(())
        })
        .unwrap();
    let post = post_cell.into_inner().unwrap();

    // Set up a test database
    let (_temp_dir, repo) = setup_test_db().await;

    // Save the post
    let post_id = repo.save_post(&post).await.unwrap();

    // Retrieve the post
    let retrieved_post = repo.get_post_by_id(post_id).await.unwrap().unwrap();

    // Check that all fields match
    assert_eq!(post.title, retrieved_post.title);
    assert_eq!(post.slug, retrieved_post.slug);
    assert_eq!(post.content, retrieved_post.content);
    assert_eq!(post.excerpt, retrieved_post.excerpt);
    assert_eq!(post.author, retrieved_post.author);
    assert_eq!(post.date, retrieved_post.date);
    assert_eq!(post.image, retrieved_post.image);
    assert_eq!(post.published, retrieved_post.published);
    assert_eq!(post.featured, retrieved_post.featured);

    // Check tags (order might be different)
    assert_eq!(post.tags.len(), retrieved_post.tags.len());
    for tag in post.tags.iter() {
        assert!(
            retrieved_post
                .tags
                .iter()
                .any(|t| t.name == tag.name && t.slug == tag.slug)
        );
    }

    // Check metadata
    assert_eq!(post.metadata.len(), retrieved_post.metadata.len());
    for (key, value) in post.metadata.iter() {
        assert_eq!(Some(value), retrieved_post.metadata.get(key));
    }
}

// Property: Updating a post is idempotent
#[tokio::test]
async fn update_post_is_idempotent() {
    let mut runner = TestRunner::default();

    // Generate a random blog post
    let post_cell = RefCell::new(None);
    runner
        .run(&blog_post_strategy(), |p| {
            post_cell.replace(Some(p.clone()));
            Ok(())
        })
        .unwrap();
    let post = post_cell.into_inner().unwrap();

    // Set up a test database
    let (_temp_dir, repo) = setup_test_db().await;

    // Save the post
    let post_id = repo.save_post(&post).await.unwrap();

    // Retrieve the post
    let retrieved_post = repo.get_post_by_id(post_id).await.unwrap().unwrap();

    // Update the post (first time)
    repo.update_post(&retrieved_post).await.unwrap();

    // Retrieve the post again
    let first_update = repo.get_post_by_id(post_id).await.unwrap().unwrap();

    // Update the post again (second time)
    repo.update_post(&first_update).await.unwrap();

    // Retrieve the post one more time
    let second_update = repo.get_post_by_id(post_id).await.unwrap().unwrap();

    // Check that the post is the same after both updates
    assert_eq!(first_update.title, second_update.title);
    assert_eq!(first_update.slug, second_update.slug);
    assert_eq!(first_update.content, second_update.content);
    assert_eq!(first_update.excerpt, second_update.excerpt);
    assert_eq!(first_update.author, second_update.author);
    assert_eq!(first_update.date, second_update.date);
    assert_eq!(first_update.image, second_update.image);
    assert_eq!(first_update.published, second_update.published);
    assert_eq!(first_update.featured, second_update.featured);

    // Check tags
    assert_eq!(first_update.tags.len(), second_update.tags.len());
    for tag in first_update.tags.iter() {
        assert!(
            second_update
                .tags
                .iter()
                .any(|t| t.name == tag.name && t.slug == tag.slug)
        );
    }

    // Check metadata
    assert_eq!(first_update.metadata.len(), second_update.metadata.len());
    for (key, value) in first_update.metadata.iter() {
        assert_eq!(Some(value), second_update.metadata.get(key));
    }
}

// Property: Deleting a post makes it unretrievable
#[tokio::test]
async fn delete_post_makes_it_unretrievable() {
    let mut runner = TestRunner::default();

    // Generate a random blog post
    let post_cell = RefCell::new(None);
    runner
        .run(&blog_post_strategy(), |p| {
            post_cell.replace(Some(p.clone()));
            Ok(())
        })
        .unwrap();
    let post = post_cell.into_inner().unwrap();

    // Set up a test database
    let (_temp_dir, repo) = setup_test_db().await;

    // Save the post
    let post_id = repo.save_post(&post).await.unwrap();

    // Verify the post exists
    let retrieved_post = repo.get_post_by_id(post_id).await.unwrap();
    assert!(retrieved_post.is_some());

    // Delete the post
    repo.delete_post(post_id).await.unwrap();

    // Verify the post no longer exists
    let deleted_post = repo.get_post_by_id(post_id).await.unwrap();
    assert!(deleted_post.is_none());
}
