/*!
 * Feed generation module
 * This module provides functionality for generating RSS and Atom feeds from blog posts
 */

use crate::blog_data::BlogPost;
use atom_syndication::{
    Content, Entry as AtomEntry, Feed as AtomFeed, Link as AtomLink, Person as AtomPerson,
    Text as AtomText,
};
use chrono::{DateTime, FixedOffset, Utc};
use im::Vector;
use rss::{
    ChannelBuilder as RssChannelBuilder, Guid as RssGuid, Item as RssItem,
    ItemBuilder as RssItemBuilder,
};
use std::str::FromStr;
use tracing::{debug, instrument};

/// Configuration for feed generation
#[derive(Debug, Clone)]
pub struct FeedConfig {
    /// Title of the feed
    pub title: String,
    /// Description of the feed
    pub description: String,
    /// Link to the website
    pub link: String,
    /// Author of the feed
    pub author: String,
    /// Email of the author
    pub email: String,
    /// Language of the feed
    pub language: String,
    /// Copyright notice
    pub copyright: String,
    /// Base URL for generating item links
    pub base_url: String,
}

impl Default for FeedConfig {
    fn default() -> Self {
        Self {
            title: "Blog".to_string(),
            description: "Latest blog posts".to_string(),
            link: "https://example.com".to_string(),
            author: "Author".to_string(),
            email: "author@example.com".to_string(),
            language: "en-us".to_string(),
            copyright: "Copyright (c) 2025".to_string(),
            base_url: "https://example.com".to_string(),
        }
    }
}

/// Generate an RSS feed from blog posts
#[instrument(skip(posts, config), err)]
pub fn generate_rss_feed(
    posts: &Vector<BlogPost>,
    config: &FeedConfig,
) -> Result<String, anyhow::Error> {
    debug!("Generating RSS feed with {} posts", posts.len());

    // Create RSS items from blog posts
    let items: Vec<RssItem> = posts
        .iter()
        .map(|post| {
            let post_url = format!("{}/blog/{}", config.base_url, post.slug);

            let mut builder = RssItemBuilder::default();
            let mut item_builder = builder
                .title(post.title.clone())
                .link(post_url.clone())
                .guid(RssGuid {
                    value: post_url,
                    permalink: true,
                })
                .pub_date(format_date_for_rss(&post.date)?)
                .description(post.excerpt.clone());

            // Add author if available
            if !post.author.is_empty() {
                item_builder = item_builder.author(post.author.clone());
            }

            // Add categories (tags)
            let categories = post
                .tags
                .iter()
                .map(|tag| rss::Category {
                    name: tag.name.clone(),
                    domain: None,
                })
                .collect::<Vec<_>>();

            if !categories.is_empty() {
                item_builder = item_builder.categories(categories);
            }

            Ok(item_builder.build())
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    // Build the RSS channel
    let channel = RssChannelBuilder::default()
        .title(config.title.clone())
        .link(config.link.clone())
        .description(config.description.clone())
        .language(Some(config.language.clone()))
        .copyright(Some(config.copyright.clone()))
        .managing_editor(Some(format!("{} ({})", config.author, config.email)))
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items)
        .build();

    // Convert the channel to XML
    Ok(channel.to_string())
}

/// Generate an Atom feed from blog posts
#[instrument(skip(posts, config), err)]
pub fn generate_atom_feed(
    posts: &Vector<BlogPost>,
    config: &FeedConfig,
) -> Result<String, anyhow::Error> {
    debug!("Generating Atom feed with {} posts", posts.len());

    // Create Atom entries from blog posts
    let entries: Vec<AtomEntry> = posts
        .iter()
        .map(|post| {
            let post_url = format!("{}/blog/{}", config.base_url, post.slug);
            let post_date = parse_date(&post.date)?;

            let mut entry = AtomEntry {
                title: AtomText::plain(post.title.clone()),
                ..Default::default()
            };
            entry.id = post_url.clone();

            // Add link
            let link = AtomLink {
                href: post_url,
                rel: "alternate".to_string(),
                ..Default::default()
            };
            entry.links.push(link);

            // Add author
            let author = AtomPerson {
                name: post.author.clone(),
                ..Default::default()
            };
            entry.authors.push(author);

            // Add content
            let content = Content {
                value: Some(post.excerpt.clone()),
                content_type: Some("text".to_string()),
                ..Default::default()
            };
            entry.content = Some(content);

            // Add published and updated dates
            entry.published = Some(post_date);
            entry.updated = post_date;

            // Add categories (tags)
            entry.categories = post
                .tags
                .iter()
                .map(|tag| atom_syndication::Category {
                    term: tag.name.clone(),
                    ..Default::default()
                })
                .collect();

            Ok(entry)
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    // Build the Atom feed
    let mut feed = AtomFeed {
        title: AtomText::plain(config.title.clone()),
        ..Default::default()
    };
    feed.id = config.link.clone();

    // Add link
    let link = AtomLink {
        href: config.link.clone(),
        rel: "self".to_string(),
        ..Default::default()
    };
    feed.links.push(link);

    // Add author
    let author = AtomPerson {
        name: config.author.clone(),
        email: Some(config.email.clone()),
        ..Default::default()
    };
    feed.authors.push(author);

    // Add updated date
    feed.updated = Utc::now().into();

    // Add entries
    feed.entries = entries;

    // Convert the feed to XML
    Ok(feed.to_string())
}

/// Format a date string for RSS (RFC 2822)
fn format_date_for_rss(date_str: &str) -> Result<String, anyhow::Error> {
    let date = parse_date(date_str)?;
    Ok(date.to_rfc2822())
}

/// Parse a date string into a DateTime
fn parse_date(date_str: &str) -> Result<DateTime<FixedOffset>, anyhow::Error> {
    // Try parsing as RFC 3339
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(date);
    }

    // Try parsing as ISO 8601
    if let Ok(date) = DateTime::from_str(date_str) {
        return Ok(date);
    }

    // Try parsing as YYYY-MM-DD
    if date_str.len() >= 10
        && date_str.chars().nth(4) == Some('-')
        && date_str.chars().nth(7) == Some('-')
    {
        let date_with_time = format!("{}T00:00:00Z", &date_str[0..10]);
        if let Ok(date) = DateTime::parse_from_rfc3339(&date_with_time) {
            return Ok(date);
        }
    }

    // If all parsing attempts fail, return an error
    Err(anyhow::anyhow!("Failed to parse date: {}", date_str))
}
