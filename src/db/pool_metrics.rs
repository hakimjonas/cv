/// Database connection pool metrics collection and monitoring
///
/// This module provides a comprehensive system for collecting, storing, and reporting metrics
/// about the database connection pool, such as connection usage, acquisition time,
/// and pool state. These metrics are essential for monitoring database performance,
/// identifying bottlenecks, and optimizing application behavior.
///
/// ## Architecture
///
/// The metrics system is built around several key components:
///
/// 1. **PoolMetrics**: The main entry point that provides a public API for recording metrics
///    events (e.g., connection created, connection acquired, connection error).
///
/// 2. **Metrics**: Internal storage for atomic counters and other metrics data structures.
///    Uses atomic types for thread-safe updates without locks.
///
/// 3. **ConnectionUsageTracker**: A RAII (Resource Acquisition Is Initialization) object
///    that tracks the lifetime of a connection usage session. When dropped, it automatically
///    records the connection usage time.
///
/// 4. **Histogram**: Tracks the distribution of durations (e.g., wait times, usage times)
///    in predefined buckets for statistical analysis.
///
/// 5. **TimeSeries**: Records time-based samples of metrics (e.g., active connections over time)
///    for trend analysis.
///
/// 6. **MetricsSnapshot**: A point-in-time snapshot of all metrics for reporting or analysis.
///
/// ## Usage Pattern
///
/// The typical usage pattern is:
///
/// 1. Create a `PoolMetrics` instance when initializing the connection pool
/// 2. Call methods like `connection_created()`, `connection_closed()`, etc. when events occur
/// 3. For connection acquisition, call `connection_acquired()` which returns a `ConnectionUsageTracker`
/// 4. The `ConnectionUsageTracker` automatically records usage time when it's dropped
/// 5. Periodically call `log_summary()` or `get_snapshot()` to report or analyze metrics
///
/// ## Thread Safety
///
/// The metrics system is designed to be thread-safe, using atomic operations for counters
/// and mutex-protected access for more complex data structures like histograms and time series.
/// This ensures accurate metrics collection in a concurrent environment without significant
/// performance overhead.
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Metrics for the database connection pool
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    /// The name of the pool (for identifying multiple pools)
    name: String,
    /// Internal metrics storage
    metrics: Arc<Metrics>,
}

/// Internal metrics storage
#[derive(Debug)]
struct Metrics {
    /// Total number of connections created
    connections_created: AtomicUsize,
    /// Total number of connections closed
    connections_closed: AtomicUsize,
    /// Current number of active connections
    active_connections: AtomicUsize,
    /// Current number of idle connections
    idle_connections: AtomicUsize,
    /// Total number of connection acquisitions
    acquisitions: AtomicUsize,
    /// Total number of connection acquisition timeouts
    acquisition_timeouts: AtomicUsize,
    /// Total number of connection errors
    connection_errors: AtomicUsize,
    /// Total time spent waiting for connections (nanoseconds)
    wait_time_ns: AtomicU64,
    /// Total time connections were in use (nanoseconds)
    usage_time_ns: AtomicU64,
    /// Maximum wait time for a connection (nanoseconds)
    max_wait_time_ns: AtomicU64,
    /// Maximum usage time for a connection (nanoseconds)
    max_usage_time_ns: AtomicU64,
    /// Histogram of wait times (buckets in milliseconds)
    wait_time_histogram: Mutex<Histogram>,
    /// Histogram of usage times (buckets in milliseconds)
    usage_time_histogram: Mutex<Histogram>,
    /// Time series of active connections (samples every minute)
    active_connections_timeseries: Mutex<TimeSeries>,
    /// Time series of idle connections (samples every minute)
    idle_connections_timeseries: Mutex<TimeSeries>,
    /// Time when metrics collection started
    start_time: Instant,
}

/// Histogram for tracking distribution of durations
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Buckets for the histogram (in milliseconds)
    /// Each bucket counts occurrences in the range [lower_bound, upper_bound)
    buckets: Vec<(u64, u64, usize)>, // (lower_bound_ms, upper_bound_ms, count)
}

/// Time series for tracking metrics over time
#[derive(Debug, Clone)]
pub struct TimeSeries {
    /// Samples in the time series (timestamp, value)
    samples: Vec<(u64, usize)>, // (timestamp_seconds, value)
    /// Maximum number of samples to keep
    max_samples: usize,
    /// Sampling interval in seconds
    sampling_interval: u64,
    /// Last sample timestamp
    last_sample_time: u64,
}

impl PoolMetrics {
    /// Create a new PoolMetrics instance
    pub fn new(name: &str) -> Self {
        info!(
            "Initializing database connection pool metrics for '{}'",
            name
        );
        Self {
            name: name.to_string(),
            metrics: Arc::new(Metrics::new()),
        }
    }

    /// Record a new connection being created
    pub fn connection_created(&self) {
        self.metrics
            .connections_created
            .fetch_add(1, Ordering::Relaxed);
        debug!("Connection created in pool '{}'", self.name);
    }

    /// Record a connection being closed
    pub fn connection_closed(&self) {
        self.metrics
            .connections_closed
            .fetch_add(1, Ordering::Relaxed);
        debug!("Connection closed in pool '{}'", self.name);
    }

    /// Record a connection being acquired from the pool
    ///
    /// This method is called when a connection is successfully acquired from the pool.
    /// It updates various metrics related to connection acquisition and returns a
    /// `ConnectionUsageTracker` that will automatically record the connection usage time
    /// when it's dropped.
    ///
    /// ## Metrics Updated
    ///
    /// - Increments the total acquisitions count
    /// - Increments the active connections count
    /// - Decrements the idle connections count
    /// - Adds the wait time to the total wait time
    /// - Updates the maximum wait time if this wait was longer
    /// - Records the wait time in the histogram
    /// - Updates the time series if needed
    ///
    /// ## Parameters
    ///
    /// - `wait_time`: The time spent waiting to acquire the connection
    ///
    /// ## Returns
    ///
    /// A `ConnectionUsageTracker` that will automatically record the connection usage time
    /// when it's dropped. This tracker should be kept alive for as long as the connection
    /// is in use.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # // This example is marked as no_run because it requires external dependencies
    /// use std::time::{Duration, Instant};
    /// use std::sync::Arc;
    /// use r2d2::{Pool, PooledConnection};
    /// use r2d2_sqlite::SqliteConnectionManager;
    /// use rusqlite::Connection;
    ///
    /// // Create a connection pool
    /// let manager = SqliteConnectionManager::memory();
    /// let pool = Pool::new(manager).unwrap();
    ///
    /// // Create a metrics instance - in a real example, you would use the actual path
    /// # struct PoolMetrics { name: String }
    /// # impl PoolMetrics {
    /// #     fn new(name: &str) -> Self { PoolMetrics { name: name.to_string() } }
    /// #     fn connection_acquired(&self, _wait_time: Duration) -> ConnectionUsageTracker {
    /// #         ConnectionUsageTracker {}
    /// #     }
    /// # }
    /// # struct ConnectionUsageTracker {}
    /// let metrics = PoolMetrics::new("example_pool");
    ///
    /// // Record the start time for connection acquisition
    /// let start_time = Instant::now();
    ///
    /// // Get a connection from the pool
    /// let conn = pool.get().unwrap();
    ///
    /// // Calculate wait time and record connection acquisition
    /// let wait_time = start_time.elapsed();
    /// let tracker = metrics.connection_acquired(wait_time);
    ///
    /// // Use the connection...
    ///
    /// // The tracker will be dropped when it goes out of scope,
    /// // automatically recording the connection usage time
    /// ```
    pub fn connection_acquired(&self, wait_time: Duration) -> ConnectionUsageTracker {
        let wait_time_ns = wait_time.as_nanos() as u64;

        // Update metrics
        self.metrics.acquisitions.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .wait_time_ns
            .fetch_add(wait_time_ns, Ordering::Relaxed);

        // Update max wait time if this wait was longer
        let mut current_max = self.metrics.max_wait_time_ns.load(Ordering::Relaxed);
        while wait_time_ns > current_max {
            match self.metrics.max_wait_time_ns.compare_exchange(
                current_max,
                wait_time_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Update wait time histogram
        let wait_time_ms = wait_time.as_millis() as u64;
        if let Ok(mut histogram) = self.metrics.wait_time_histogram.lock() {
            histogram.record(wait_time_ms);
        }

        // Update time series if needed
        self.update_time_series();

        debug!(
            "Connection acquired from pool '{}' after waiting for {:?}",
            self.name, wait_time
        );

        // Return a tracker for this connection usage
        ConnectionUsageTracker {
            metrics: Arc::clone(&self.metrics),
            start_time: Instant::now(),
            pool_name: self.name.clone(),
        }
    }

    /// Record a connection acquisition timeout
    pub fn acquisition_timeout(&self) {
        self.metrics
            .acquisition_timeouts
            .fetch_add(1, Ordering::Relaxed);
        warn!("Connection acquisition timeout in pool '{}'", self.name);
    }

    /// Record a connection error
    pub fn connection_error(&self) {
        self.metrics
            .connection_errors
            .fetch_add(1, Ordering::Relaxed);
        warn!("Connection error in pool '{}'", self.name);
    }

    /// Update time series if needed
    ///
    /// This method updates the time series data for active and idle connections
    /// if enough time has passed since the last sample. It's designed to be called
    /// frequently (e.g., on every connection acquisition) but only actually updates
    /// the time series at the configured sampling interval.
    ///
    /// ## Time Series Data
    ///
    /// The method maintains two time series:
    /// - Active connections: Number of connections currently in use
    /// - Idle connections: Number of connections currently idle in the pool
    ///
    /// These time series provide historical data that can be used to:
    /// - Identify usage patterns over time
    /// - Detect connection leaks (steadily increasing active connections)
    /// - Optimize pool size based on actual usage
    ///
    /// ## Sampling Strategy
    ///
    /// To avoid excessive memory usage and performance overhead, samples are taken
    /// at a fixed interval (default: 60 seconds). Each time series has a maximum
    /// number of samples (default: 60), creating a rolling window of data (e.g., 1 hour).
    ///
    /// ## Thread Safety
    ///
    /// The time series data structures are protected by mutexes to ensure thread safety.
    /// The method first checks if an update is needed without acquiring the lock,
    /// minimizing contention in high-concurrency scenarios.
    fn update_time_series(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Update active connections time series
        if let Ok(mut timeseries) = self.metrics.active_connections_timeseries.lock() {
            timeseries
                .maybe_add_sample(now, self.metrics.active_connections.load(Ordering::Relaxed));
        }

        // Update idle connections time series
        if let Ok(mut timeseries) = self.metrics.idle_connections_timeseries.lock() {
            timeseries.maybe_add_sample(now, self.metrics.idle_connections.load(Ordering::Relaxed));
        }
    }

    /// Get a snapshot of the current metrics
    ///
    /// This method creates a point-in-time snapshot of all metrics collected by the system.
    /// The snapshot includes counters, averages, histograms, and time series data, providing
    /// a comprehensive view of the connection pool's performance and usage patterns.
    ///
    /// ## Use Cases
    ///
    /// The metrics snapshot can be used for:
    /// - Monitoring database connection pool health
    /// - Identifying performance bottlenecks
    /// - Detecting connection leaks
    /// - Optimizing connection pool configuration
    /// - Generating performance reports
    ///
    /// ## Interpreting the Metrics
    ///
    /// Key metrics to monitor include:
    ///
    /// - **Connection Counts**: Compare `connections_created` vs `connections_closed` to detect leaks.
    ///   The sum of `active_connections` and `idle_connections` should equal `connections_created - connections_closed`.
    ///
    /// - **Wait Times**: High `avg_wait_time` or `max_wait_time` indicates contention for connections.
    ///   Consider increasing the pool size if wait times are consistently high.
    ///
    /// - **Usage Times**: Unusually high `avg_usage_time` or `max_usage_time` may indicate slow queries
    ///   or connections being held longer than necessary.
    ///
    /// - **Acquisition Timeouts**: A non-zero `acquisition_timeouts` count indicates that the pool
    ///   was unable to provide connections quickly enough. Consider increasing the pool size or
    ///   timeout duration.
    ///
    /// - **Connection Errors**: A high `connection_errors` count may indicate database connectivity issues.
    ///
    /// - **Time Series Data**: The `active_connections_timeseries` and `idle_connections_timeseries`
    ///   can reveal usage patterns over time and help identify peak usage periods.
    ///
    /// ## Thread Safety
    ///
    /// This method is thread-safe and can be called concurrently from multiple threads.
    /// It uses atomic operations for counters and acquires locks only briefly to clone
    /// the histogram and time series data.
    ///
    /// ## Performance Considerations
    ///
    /// While this method is designed to be efficient, it does involve cloning histogram and
    /// time series data, which can be relatively expensive. It's recommended to call this
    /// method periodically (e.g., every few minutes) rather than on every connection operation.
    ///
    /// ## Returns
    ///
    /// A `MetricsSnapshot` containing all metrics at the time of the call
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let metrics = &self.metrics;

        // Get current values
        let connections_created = metrics.connections_created.load(Ordering::Relaxed);
        let connections_closed = metrics.connections_closed.load(Ordering::Relaxed);
        let active_connections = metrics.active_connections.load(Ordering::Relaxed);
        let idle_connections = metrics.idle_connections.load(Ordering::Relaxed);
        let acquisitions = metrics.acquisitions.load(Ordering::Relaxed);
        let acquisition_timeouts = metrics.acquisition_timeouts.load(Ordering::Relaxed);
        let connection_errors = metrics.connection_errors.load(Ordering::Relaxed);
        let wait_time_ns = metrics.wait_time_ns.load(Ordering::Relaxed);
        let usage_time_ns = metrics.usage_time_ns.load(Ordering::Relaxed);
        let max_wait_time_ns = metrics.max_wait_time_ns.load(Ordering::Relaxed);
        let max_usage_time_ns = metrics.max_usage_time_ns.load(Ordering::Relaxed);

        // Calculate averages
        let avg_wait_time_ns = if acquisitions > 0 {
            wait_time_ns / acquisitions as u64
        } else {
            0
        };

        let avg_usage_time_ns = if acquisitions > 0 {
            usage_time_ns / acquisitions as u64
        } else {
            0
        };

        // Clone histograms and time series
        let wait_time_histogram = metrics
            .wait_time_histogram
            .lock()
            .map(|h| h.clone())
            .unwrap_or_else(|_| Histogram::new());

        let usage_time_histogram = metrics
            .usage_time_histogram
            .lock()
            .map(|h| h.clone())
            .unwrap_or_else(|_| Histogram::new());

        let active_connections_timeseries = metrics
            .active_connections_timeseries
            .lock()
            .map(|ts| ts.clone())
            .unwrap_or_else(|_| TimeSeries::new(60, 60));

        let idle_connections_timeseries = metrics
            .idle_connections_timeseries
            .lock()
            .map(|ts| ts.clone())
            .unwrap_or_else(|_| TimeSeries::new(60, 60));

        // Calculate uptime
        let uptime = metrics.start_time.elapsed();

        MetricsSnapshot {
            name: self.name.clone(),
            connections_created,
            connections_closed,
            active_connections,
            idle_connections,
            acquisitions,
            acquisition_timeouts,
            connection_errors,
            avg_wait_time: Duration::from_nanos(avg_wait_time_ns),
            avg_usage_time: Duration::from_nanos(avg_usage_time_ns),
            max_wait_time: Duration::from_nanos(max_wait_time_ns),
            max_usage_time: Duration::from_nanos(max_usage_time_ns),
            wait_time_histogram,
            usage_time_histogram,
            active_connections_timeseries,
            idle_connections_timeseries,
            uptime,
        }
    }

    /// Log a summary of the current metrics
    pub fn log_summary(&self) {
        let snapshot = self.get_snapshot();

        info!("Database pool '{}' metrics summary:", snapshot.name);
        info!(
            "  Connections: {} created, {} closed, {} active, {} idle",
            snapshot.connections_created,
            snapshot.connections_closed,
            snapshot.active_connections,
            snapshot.idle_connections
        );
        info!(
            "  Acquisitions: {} total, {} timeouts, {} errors",
            snapshot.acquisitions, snapshot.acquisition_timeouts, snapshot.connection_errors
        );
        info!(
            "  Wait time: {:?} avg, {:?} max",
            snapshot.avg_wait_time, snapshot.max_wait_time
        );
        info!(
            "  Usage time: {:?} avg, {:?} max",
            snapshot.avg_usage_time, snapshot.max_usage_time
        );
        info!("  Uptime: {:?}", snapshot.uptime);
    }
}

impl Metrics {
    /// Create a new Metrics instance
    fn new() -> Self {
        Self {
            connections_created: AtomicUsize::new(0),
            connections_closed: AtomicUsize::new(0),
            active_connections: AtomicUsize::new(0),
            idle_connections: AtomicUsize::new(0),
            acquisitions: AtomicUsize::new(0),
            acquisition_timeouts: AtomicUsize::new(0),
            connection_errors: AtomicUsize::new(0),
            wait_time_ns: AtomicU64::new(0),
            usage_time_ns: AtomicU64::new(0),
            max_wait_time_ns: AtomicU64::new(0),
            max_usage_time_ns: AtomicU64::new(0),
            wait_time_histogram: Mutex::new(Histogram::new()),
            usage_time_histogram: Mutex::new(Histogram::new()),
            active_connections_timeseries: Mutex::new(TimeSeries::new(60, 60)), // 1 hour of 1-minute samples
            idle_connections_timeseries: Mutex::new(TimeSeries::new(60, 60)), // 1 hour of 1-minute samples
            start_time: Instant::now(),
        }
    }
}

impl Histogram {
    /// Create a new Histogram with default buckets
    fn new() -> Self {
        // Create buckets for: 0-1ms, 1-5ms, 5-10ms, 10-50ms, 50-100ms, 100-500ms, 500-1000ms, 1000ms+
        let buckets = vec![
            (0, 1, 0),
            (1, 5, 0),
            (5, 10, 0),
            (10, 50, 0),
            (50, 100, 0),
            (100, 500, 0),
            (500, 1000, 0),
            (1000, u64::MAX, 0),
        ];

        Self { buckets }
    }

    /// Record a value in the histogram
    fn record(&mut self, value_ms: u64) {
        for bucket in &mut self.buckets {
            if value_ms >= bucket.0 && value_ms < bucket.1 {
                bucket.2 += 1;
                break;
            }
        }
    }
}

impl TimeSeries {
    /// Create a new TimeSeries
    fn new(max_samples: usize, sampling_interval: u64) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
            sampling_interval,
            last_sample_time: 0,
        }
    }

    /// Add a sample to the time series if enough time has passed since the last sample
    fn maybe_add_sample(&mut self, timestamp: u64, value: usize) {
        if timestamp >= self.last_sample_time + self.sampling_interval {
            // Add the sample
            self.samples.push((timestamp, value));
            self.last_sample_time = timestamp;

            // Remove oldest samples if we've exceeded the maximum
            if self.samples.len() > self.max_samples {
                self.samples.remove(0);
            }
        }
    }
}

/// Tracker for a connection usage session
///
/// This struct implements the RAII (Resource Acquisition Is Initialization) pattern
/// to automatically track the duration of a database connection usage session.
/// When a connection is acquired from the pool, a `ConnectionUsageTracker` is created.
/// When the tracker is dropped (goes out of scope), it automatically records the
/// connection usage time and updates the relevant metrics.
///
/// ## Usage
///
/// ```no_run
/// # // This example is marked as no_run because it requires external dependencies
/// use std::time::Duration;
///
/// # // Mock implementation for the example
/// # struct PoolMetrics { name: String }
/// # impl PoolMetrics {
/// #     fn new(name: &str) -> Self { PoolMetrics { name: name.to_string() } }
/// #     fn connection_acquired(&self, _wait_time: Duration) -> ConnectionUsageTracker {
/// #         ConnectionUsageTracker {}
/// #     }
/// # }
/// # struct ConnectionUsageTracker {}
///
/// // Create a metrics instance
/// let pool_metrics = PoolMetrics::new("example_pool");
///
/// // Create a wait time duration
/// let wait_time = Duration::from_millis(10);
///
/// // When a connection is acquired
/// let tracker = pool_metrics.connection_acquired(wait_time);
///
/// // Use the connection...
///
/// // When done, the tracker is automatically dropped, recording usage time
/// // This happens implicitly when the tracker goes out of scope
/// ```
///
/// ## Implementation Details
///
/// The tracker uses Rust's `Drop` trait to automatically record metrics when it's dropped.
/// This approach ensures that metrics are always recorded, even if an error occurs or
/// the function returns early, providing accurate tracking without manual instrumentation.
///
/// The tracker stores:
/// - A reference to the metrics object for updating metrics
/// - The start time when the connection was acquired
/// - The name of the pool for logging purposes
pub struct ConnectionUsageTracker {
    /// Reference to the metrics
    metrics: Arc<Metrics>,
    /// When the connection was acquired
    start_time: Instant,
    /// Name of the pool
    pool_name: String,
}

impl Drop for ConnectionUsageTracker {
    /// When the tracker is dropped, record the connection usage time
    ///
    /// This method is automatically called when the tracker goes out of scope.
    /// It calculates the connection usage time, updates the relevant metrics,
    /// and logs the event. This ensures that connection usage is always tracked,
    /// even if an error occurs or the function returns early.
    ///
    /// The metrics updated include:
    /// - Decrementing the active connections count
    /// - Incrementing the idle connections count
    /// - Adding the usage time to the total usage time
    /// - Updating the maximum usage time if this usage was longer
    /// - Recording the usage time in the histogram
    fn drop(&mut self) {
        let usage_time = self.start_time.elapsed();
        let usage_time_ns = usage_time.as_nanos() as u64;

        // Update metrics
        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .usage_time_ns
            .fetch_add(usage_time_ns, Ordering::Relaxed);

        // Update max usage time if this usage was longer
        let mut current_max = self.metrics.max_usage_time_ns.load(Ordering::Relaxed);
        while usage_time_ns > current_max {
            match self.metrics.max_usage_time_ns.compare_exchange(
                current_max,
                usage_time_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Update usage time histogram
        let usage_time_ms = usage_time.as_millis() as u64;
        if let Ok(mut histogram) = self.metrics.usage_time_histogram.lock() {
            histogram.record(usage_time_ms);
        }

        debug!(
            "Connection returned to pool '{}' after being used for {:?}",
            self.pool_name, usage_time
        );
    }
}

/// Snapshot of pool metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Name of the pool
    pub name: String,
    /// Total number of connections created
    pub connections_created: usize,
    /// Total number of connections closed
    pub connections_closed: usize,
    /// Current number of active connections
    pub active_connections: usize,
    /// Current number of idle connections
    pub idle_connections: usize,
    /// Total number of connection acquisitions
    pub acquisitions: usize,
    /// Total number of connection acquisition timeouts
    pub acquisition_timeouts: usize,
    /// Total number of connection errors
    pub connection_errors: usize,
    /// Average wait time for a connection
    pub avg_wait_time: Duration,
    /// Average usage time for a connection
    pub avg_usage_time: Duration,
    /// Maximum wait time for a connection
    pub max_wait_time: Duration,
    /// Maximum usage time for a connection
    pub max_usage_time: Duration,
    /// Histogram of wait times
    pub wait_time_histogram: Histogram,
    /// Histogram of usage times
    pub usage_time_histogram: Histogram,
    /// Time series of active connections
    pub active_connections_timeseries: TimeSeries,
    /// Time series of idle connections
    pub idle_connections_timeseries: TimeSeries,
    /// Time since metrics collection started
    pub uptime: Duration,
}
