use anyhow::{Context, Result};
use std::path::Path;

use super::{ColorPalette, ColorSchemeProvider};

/// Cache wrapper for any provider
pub struct CachedProvider<P: ColorSchemeProvider> {
    provider: P,
    cache_dir: String,
}

impl<P: ColorSchemeProvider> CachedProvider<P> {
    pub fn new(provider: P, cache_dir: &str) -> Self {
        CachedProvider {
            provider,
            cache_dir: cache_dir.to_string(),
        }
    }
}

impl<P: ColorSchemeProvider> ColorSchemeProvider for CachedProvider<P> {
    fn fetch(&self, name: &str, variant: Option<&str>) -> Result<ColorPalette> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&self.cache_dir).context("Failed to create cache directory")?;

        // Create a cache key that includes the variant
        let variant_suffix = variant.unwrap_or("default");
        let cache_key = format!("{}-{}", name, variant_suffix);
        let cache_file = Path::new(&self.cache_dir).join(format!("{}.json", cache_key));

        // Try to load from cache
        if cache_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&cache_file) {
                if let Ok(palette) = serde_json::from_str(&content) {
                    println!("✅ Using cached color scheme: {}", name);
                    return Ok(palette);
                }
            }
        }

        // Fetch from provider
        println!(
            "🌐 Fetching color scheme from {}: {}",
            self.provider.provider_name(),
            name
        );
        let palette = self.provider.fetch(name, variant)?;

        // Save to cache
        let json = serde_json::to_string_pretty(&palette)?;
        std::fs::write(&cache_file, json).context("Failed to write cache file")?;

        Ok(palette)
    }

    fn list_available(&self) -> Result<Vec<String>> {
        self.provider.list_available()
    }

    fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }
}
