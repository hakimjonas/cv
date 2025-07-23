/// End-to-End tests for the Blog API
///
/// This file contains end-to-end tests for the Blog API using fantoccini,
/// a high-level API for controlling headless browsers through WebDriver.
///
/// To run these tests, you need to have a WebDriver server running, such as
/// ChromeDriver or GeckoDriver, and the blog API server running on localhost:3000.
///
/// ```bash
/// # Start the blog API server
/// cargo run --bin blog_api_server
///
/// # In another terminal, start ChromeDriver
/// chromedriver --port=4444
///
/// # Run the tests
/// cargo test --test blog_api_e2e_test
/// ```
use anyhow::{Context, Result};
use fantoccini::{Client, ClientBuilder};
use serde_json::json;
use std::time::Duration;

/// Create a new WebDriver client
async fn create_client() -> Result<Client> {
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .context("Failed to connect to WebDriver")?;
    
    Ok(client)
}

/// Test creating a blog post through the API documentation UI
#[tokio::test]
async fn test_create_blog_post_through_swagger_ui() -> Result<()> {
    // Create a new WebDriver client
    let mut client = create_client().await?;
    
    // Navigate to the Swagger UI
    client.goto("http://localhost:3000/api-docs").await?;
    
    // Wait for the page to load
    client.wait().for_element("h2.title").await?;
    
    // Find and expand the POST /api/blog endpoint
    let post_endpoint = client.find(r#"[data-path="/api/blog"][data-method="post"]"#).await?;
    post_endpoint.click().await?;
    
    // Click the "Try it out" button
    let try_it_out_button = client.find(".try-out__btn").await?;
    try_it_out_button.click().await?;
    
    // Enter the request body
    let request_body = json!({
        "title": "E2E Test Post",
        "slug": "e2e-test-post",
        "date": "2025-07-23",
        "author": "E2E Test",
        "excerpt": "This is a test post created by an end-to-end test",
        "content": "# E2E Test Post\n\nThis post was created by an automated end-to-end test.",
        "published": true,
        "featured": false,
        "tags": [
            {
                "name": "Test",
                "slug": "test"
            }
        ]
    }).to_string();
    
    let request_body_textarea = client.find(".body-param__text").await?;
    request_body_textarea.clear().await?;
    request_body_textarea.send_keys(&request_body).await?;
    
    // Click the "Execute" button
    let execute_button = client.find(".execute").await?;
    execute_button.click().await?;
    
    // Wait for the response
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check the response status code
    let response_status = client.find(".response-col_status").await?;
    let status_text = response_status.text().await?;
    assert!(status_text.contains("201") || status_text.contains("200"), "Expected status 201 or 200, got {}", status_text);
    
    // Check the response body
    let response_body = client.find(".response-col_description__inner").await?;
    let body_text = response_body.text().await?;
    assert!(body_text.contains("E2E Test Post"), "Response body does not contain the post title");
    
    // Clean up by deleting the post
    // Navigate to the DELETE endpoint
    client.goto("http://localhost:3000/api-docs").await?;
    let delete_endpoint = client.find(r#"[data-path="/api/blog/{slug}"][data-method="delete"]"#).await?;
    delete_endpoint.click().await?;
    
    // Click the "Try it out" button
    let try_it_out_button = client.find(".try-out__btn").await?;
    try_it_out_button.click().await?;
    
    // Enter the slug
    let slug_input = client.find("input[placeholder='slug']").await?;
    slug_input.clear().await?;
    slug_input.send_keys("e2e-test-post").await?;
    
    // Click the "Execute" button
    let execute_button = client.find(".execute").await?;
    execute_button.click().await?;
    
    // Wait for the response
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check the response status code
    let response_status = client.find(".response-col_status").await?;
    let status_text = response_status.text().await?;
    assert!(status_text.contains("204") || status_text.contains("200"), "Expected status 204 or 200, got {}", status_text);
    
    // Close the client
    client.close().await?;
    
    Ok(())
}

/// Test viewing the documentation page
#[tokio::test]
async fn test_view_documentation_page() -> Result<()> {
    // Create a new WebDriver client
    let mut client = create_client().await?;
    
    // Navigate to the documentation page
    client.goto("http://localhost:3000/docs").await?;
    
    // Wait for the page to load
    client.wait().for_element("h1").await?;
    
    // Check that the page title is correct
    let title = client.find("h1").await?;
    let title_text = title.text().await?;
    assert_eq!(title_text, "CV and Blog Application Documentation", "Page title is incorrect");
    
    // Check that the documentation resources section exists
    let resources_heading = client.find("h2").await?;
    let resources_text = resources_heading.text().await?;
    assert_eq!(resources_text, "Documentation Resources", "Documentation resources heading is incorrect");
    
    // Check that the API Guide link exists
    let api_guide_link = client.find("a[href='/API_GUIDE.md']").await?;
    let api_guide_text = api_guide_link.text().await?;
    assert_eq!(api_guide_text, "View API Guide", "API Guide link text is incorrect");
    
    // Close the client
    client.close().await?;
    
    Ok(())
}

/// Test the blog API directly using reqwest
#[tokio::test]
async fn test_blog_api_directly() -> Result<()> {
    use reqwest::Client;
    
    // Create a new HTTP client
    let client = Client::new();
    
    // Create a new blog post
    let post = json!({
        "title": "Direct API Test Post",
        "slug": "direct-api-test-post",
        "date": "2025-07-23",
        "author": "API Test",
        "excerpt": "This is a test post created by a direct API test",
        "content": "# Direct API Test Post\n\nThis post was created by an automated API test.",
        "published": true,
        "featured": false,
        "tags": [
            {
                "name": "Test",
                "slug": "test"
            }
        ]
    });
    
    // Send the POST request
    let response = client.post("http://localhost:3000/api/blog")
        .json(&post)
        .send()
        .await
        .context("Failed to send POST request")?;
    
    // Check the response status code
    assert!(response.status().is_success(), "POST request failed with status {}", response.status());
    
    // Get the created post
    let response = client.get("http://localhost:3000/api/blog/direct-api-test-post")
        .send()
        .await
        .context("Failed to send GET request")?;
    
    // Check the response status code
    assert!(response.status().is_success(), "GET request failed with status {}", response.status());
    
    // Parse the response body
    let post_response: serde_json::Value = response.json().await?;
    
    // Check that the post title is correct
    assert_eq!(post_response["title"], "Direct API Test Post", "Post title is incorrect");
    
    // Delete the post
    let response = client.delete("http://localhost:3000/api/blog/direct-api-test-post")
        .send()
        .await
        .context("Failed to send DELETE request")?;
    
    // Check the response status code
    assert!(response.status().is_success(), "DELETE request failed with status {}", response.status());
    
    Ok(())
}