//! Runtime module for managing async runtime in a functional way
//!
//! This module provides a way to access a shared Tokio runtime
//! without violating functional programming principles.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Shared Tokio runtime wrapped in Arc for thread-safety
static RUNTIME: Lazy<Arc<Runtime>> =
    Lazy::new(|| Arc::new(Runtime::new().expect("Failed to create Tokio runtime")));

/// Executes an async function in the shared runtime
///
/// # Arguments
///
/// * `future` - The async function to execute
///
/// # Returns
///
/// A Result containing the return value of the async function
///
/// # Example
///
/// ```
/// use crate::runtime;
///
/// let result = runtime::run_async(async {
///     // Some async operation
///     Ok(42)
/// })?;
/// ```
pub fn run_async<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    RUNTIME
        .block_on(future)
        .context("Failed to execute async function")
}

/// Gets a reference to the shared runtime
///
/// This is useful when you need direct access to the runtime
/// for more complex scenarios.
///
/// # Returns
///
/// An Arc-wrapped reference to the Tokio runtime
pub fn get_runtime() -> Arc<Runtime> {
    RUNTIME.clone()
}
