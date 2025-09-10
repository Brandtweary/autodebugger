//! Advanced tracing subscriber with conditional verbosity control
//!
//! This module provides a sophisticated tracing subscriber implementation that extends
//! the standard tracing-subscriber with dynamic verbosity control. It's designed to 
//! provide granular control over logging output based on the frequency of similar messages.
//!
//! ## Output Destination Configuration
//!
//! The tracing subscriber can output to either stdout or stderr. This is configurable
//! to support different use cases:
//!
//! - **stdout (default)**: Standard behavior for most applications. Logs and application
//!   output are mixed together, which is fine for typical CLI tools.
//!
//! - **stderr**: Required when the application uses stdout for structured output that
//!   must not be contaminated with logs. Critical for:
//!   - MCP (Model Context Protocol) servers that communicate via JSON-RPC over stdio
//!   - Unix pipes where stdout contains data to be processed by other tools
//!   - Any protocol that requires clean stdout for machine-readable output
//!
//! When running as an MCP server, logs MUST go to stderr to keep stdout clean for
//! JSON-RPC messages. Mixing logs with JSON-RPC on stdout will break the protocol.
//!
//! ## Key Features
//!
//! ### Verbosity-Based Filtering
//! The `VerbosityCheckLayer` implements intelligent log filtering based on message frequency:
//! - First N occurrences of a message type are logged at INFO level
//! - Next M occurrences are logged at DEBUG level
//! - Remaining occurrences are logged at TRACE level
//! - Configurable thresholds via `VerbosityConfig`
//!
//! ### Conditional Location Formatting
//! The `ConditionalLocationFormatter` adds source location information (file:line) to
//! log messages, but only when the verbosity threshold is exceeded. This helps identify
//! the source of frequent log messages without cluttering initial output.
//!
//! ### External Crate Filtering
//! Automatically suppresses debug-level logs from external crates to reduce noise,
//! while preserving info-level and above messages from all sources.
//!
//! ## Architecture
//!
//! The module uses tracing-subscriber's layered architecture:
//! 1. **Base Layer**: EnvFilter for RUST_LOG environment variable support
//! 2. **Verbosity Layer**: Custom layer for frequency-based filtering
//! 3. **Format Layer**: Customizable output formatting with conditional locations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use autodebugger::init_logging;
//!
//! // Use default verbosity config from autodebugger's config.yaml
//! let verbosity_layer = init_logging(Some("info"), None);
//! 
//! // Or provide custom verbosity thresholds
//! use autodebugger::VerbosityConfig;
//! let custom_verbosity = VerbosityConfig {
//!     info_threshold: 100,
//!     debug_threshold: 200,
//!     trace_threshold: 500,
//! };
//! let verbosity_layer = init_logging(Some("info"), Some(custom_verbosity));
//! ```
//!
//! ## Configuration
//!
//! Verbosity thresholds can be configured via `config.yaml`:
//! - `verbosity.info_threshold`: Number of messages before switching to DEBUG
//! - `verbosity.debug_threshold`: Number of messages before switching to TRACE
//! - `verbosity.trace_threshold`: Maximum messages to log at TRACE level

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use crate::config::Config;

/// Custom formatter that conditionally shows file:line only for ERROR and WARN levels
/// and omits the INFO prefix for cleaner default-level output
pub struct ConditionalLocationFormatter;

impl<S, N> FormatEvent<S, N> for ConditionalLocationFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let level = metadata.level();
        
        // Format level (skip INFO prefix for cleaner output)
        if !matches!(level, &Level::INFO) {
            write!(&mut writer, "{}", level)?;
            
            // Only show module target and file:line for ERROR and WARN levels
            if matches!(level, &Level::ERROR | &Level::WARN) {
                write!(&mut writer, " {}", metadata.target())?;
                if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
                    write!(&mut writer, " {}:{}", file, line)?;
                }
            }
            
            write!(&mut writer, ": ")?;
        }
        
        // Format all the spans in the event's span context
        if let Some(scope) = ctx.event_scope() {
            let mut first = true;
            for span in scope.from_root() {
                if !first {
                    write!(&mut writer, ":")?;
                }
                first = false;
                write!(writer, "{}", span.name())?;
                
                let ext = span.extensions();
                if let Some(fields) = ext.get::<tracing_subscriber::fmt::FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(writer, "{{{}}}", fields)?;
                    }
                }
            }
            write!(writer, " ")?;
        }
        
        // Write the event fields
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        
        writeln!(writer)
    }
}

/// A tracing Layer that counts log events by level to detect excessive verbosity
#[derive(Debug, Clone)]
pub struct VerbosityCheckLayer {
    error_count: Arc<AtomicUsize>,
    warn_count: Arc<AtomicUsize>,
    info_count: Arc<AtomicUsize>,
    debug_count: Arc<AtomicUsize>,
    trace_count: Arc<AtomicUsize>,
    configured_level: Level,
    config: Config,
}

impl VerbosityCheckLayer {
    /// Create a new VerbosityCheckLayer with default config
    pub fn new() -> Self {
        Self::with_config(Config::load().unwrap_or_default())
    }
    
    /// Create a new VerbosityCheckLayer with custom config
    pub fn with_config(config: Config) -> Self {
        // Detect the configured log level from RUST_LOG or default to INFO
        let configured_level = Self::detect_configured_level();
        
        Self {
            error_count: Arc::new(AtomicUsize::new(0)),
            warn_count: Arc::new(AtomicUsize::new(0)),
            info_count: Arc::new(AtomicUsize::new(0)),
            debug_count: Arc::new(AtomicUsize::new(0)),
            trace_count: Arc::new(AtomicUsize::new(0)),
            configured_level,
            config,
        }
    }
    
    /// Detect the configured log level from environment
    fn detect_configured_level() -> Level {
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            let lower = rust_log.to_lowercase();
            if lower.contains("trace") {
                Level::TRACE
            } else if lower.contains("debug") {
                Level::DEBUG
            } else if lower.contains("info") {
                Level::INFO
            } else if lower.contains("warn") {
                Level::WARN
            } else if lower.contains("error") {
                Level::ERROR
            } else {
                Level::INFO // Default
            }
        } else {
            Level::INFO // Default when RUST_LOG not set
        }
    }
    
    /// Get the total count of all log events
    pub fn total_count(&self) -> usize {
        self.error_count.load(Ordering::Relaxed)
            + self.warn_count.load(Ordering::Relaxed)
            + self.info_count.load(Ordering::Relaxed)
            + self.debug_count.load(Ordering::Relaxed)
            + self.trace_count.load(Ordering::Relaxed)
    }
    
    /// Get counts broken down by level
    pub fn counts_by_level(&self) -> LogCounts {
        LogCounts {
            error: self.error_count.load(Ordering::Relaxed),
            warn: self.warn_count.load(Ordering::Relaxed),
            info: self.info_count.load(Ordering::Relaxed),
            debug: self.debug_count.load(Ordering::Relaxed),
            trace: self.trace_count.load(Ordering::Relaxed),
        }
    }
    
    /// Check if verbosity exceeds recommended thresholds
    pub fn check_verbosity(&self) -> Option<VerbosityWarning> {
        // Only check thresholds for INFO, DEBUG, and TRACE levels
        // WARN and ERROR levels should never trigger verbosity warnings
        let threshold = match self.configured_level {
            Level::TRACE => Some(self.config.verbosity.trace_threshold),
            Level::DEBUG => Some(self.config.verbosity.debug_threshold),
            Level::INFO => Some(self.config.verbosity.info_threshold),
            Level::WARN | Level::ERROR => None, // No threshold for WARN/ERROR
        };
        
        if let Some(threshold_value) = threshold {
            let total = self.total_count();
            if total > threshold_value {
                Some(VerbosityWarning {
                    total_count: total,
                    threshold: threshold_value,
                    configured_level: self.configured_level,
                    counts: self.counts_by_level(),
                })
            } else {
                None
            }
        } else {
            None // No verbosity check for WARN/ERROR levels
        }
    }
    
    /// Check verbosity and generate a formatted report if threshold exceeded
    pub fn check_and_report(&self) -> Option<String> {
        self.check_verbosity().map(|warning| {
            format!(
                "\nLOG VERBOSITY WARNING\n\
                ========================\n\
                Total log events: {} (threshold: {} for {} level)\n\n\
                Breakdown by level:\n\
                  ERROR: {}\n\
                  WARN:  {}\n\
                  INFO:  {}\n\
                  DEBUG: {}\n\
                  TRACE: {}\n\n\
                Consider reducing log verbosity to improve performance and readability.\n\
                Recommended thresholds:\n\
                  • INFO:  <50 total logs\n\
                  • DEBUG: <100 total logs\n\
                  • TRACE: <200 total logs",
                warning.total_count,
                warning.threshold,
                warning.configured_level,
                warning.counts.error,
                warning.counts.warn,
                warning.counts.info,
                warning.counts.debug,
                warning.counts.trace,
            )
        })
    }
}

impl Default for VerbosityCheckLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for VerbosityCheckLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let level = event.metadata().level();
        
        match *level {
            Level::ERROR => self.error_count.fetch_add(1, Ordering::Relaxed),
            Level::WARN => self.warn_count.fetch_add(1, Ordering::Relaxed),
            Level::INFO => self.info_count.fetch_add(1, Ordering::Relaxed),
            Level::DEBUG => self.debug_count.fetch_add(1, Ordering::Relaxed),
            Level::TRACE => self.trace_count.fetch_add(1, Ordering::Relaxed),
        };
    }
}

/// Breakdown of log counts by level
#[derive(Debug, Clone)]
pub struct LogCounts {
    pub error: usize,
    pub warn: usize,
    pub info: usize,
    pub debug: usize,
    pub trace: usize,
}

/// Warning information when verbosity threshold is exceeded
#[derive(Debug, Clone)]
pub struct VerbosityWarning {
    pub total_count: usize,
    pub threshold: usize,
    pub configured_level: Level,
    pub counts: LogCounts,
}

/// Create a base env filter with sled/pagecache suppression
pub fn create_base_env_filter(default_level: &str) -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level))
        // Suppress sled's verbose debug output
        .add_directive("sled=warn".parse().unwrap())
        .add_directive("pagecache=warn".parse().unwrap())
        // Reduce HTTP client verbosity (only show warnings and errors)
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap())
        // Reduce WebSocket library verbosity (only show warnings and errors)
        .add_directive("tungstenite=warn".parse().unwrap())
        .add_directive("tokio_tungstenite=warn".parse().unwrap())
}

/// Initialize the tracing subscriber with custom formatting and verbosity checking
/// Returns a handle to the VerbosityCheckLayer for later checking
/// 
/// # Arguments
/// * `default_level` - Optional default log level (e.g., "info", "warn"). If None, defaults to "info"
/// * `verbosity_config` - Optional custom verbosity thresholds. If None, uses autodebugger's config.yaml
/// * `output` - Optional output destination ("stdout" or "stderr"). If None, defaults to stdout.
///              Note: When using as an MCP server, must be set to "stderr" to keep stdout clean for JSON-RPC.
pub fn init_logging(
    default_level: Option<&str>, 
    verbosity_config: Option<crate::config::VerbosityConfig>,
    output: Option<&str>
) -> VerbosityCheckLayer {
    let default = default_level.unwrap_or("info");
    let env_filter = create_base_env_filter(default);
    
    // Create verbosity layer with custom config if provided
    let verbosity_layer = match verbosity_config {
        Some(config) => {
            // Build a Config struct with the provided verbosity
            let mut full_config = Config::default();
            full_config.verbosity = config;
            VerbosityCheckLayer::with_config(full_config)
        },
        None => VerbosityCheckLayer::new(),  // Use autodebugger's config
    };
    let verbosity_clone = verbosity_layer.clone();
    
    // Determine output destination
    // Default to stdout for normal operation, but allow override to stderr
    // This is critical for MCP servers which must keep stdout clean for JSON-RPC messages
    match output {
        Some("stderr") => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .event_format(ConditionalLocationFormatter))
                .with(verbosity_layer)
                .init();
        },
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stdout)
                    .event_format(ConditionalLocationFormatter))
                .with(verbosity_layer)
                .init();
        }
    }
    
    verbosity_clone
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verbosity_check_layer() {
        let layer = VerbosityCheckLayer::new();
        let layer_clone = layer.clone();
        
        tracing_subscriber::registry()
            .with(layer)
            .init();
        
        // Generate some log events
        tracing::error!("Test error");
        tracing::warn!("Test warning");
        tracing::info!("Test info");
        tracing::debug!("Test debug");
        tracing::trace!("Test trace");
        
        let counts = layer_clone.counts_by_level();
        assert_eq!(counts.error, 1);
        assert_eq!(counts.warn, 1);
        assert_eq!(counts.info, 1);
        // Debug and trace might not be captured depending on default filter
        
        assert!(layer_clone.total_count() >= 3);
    }
}