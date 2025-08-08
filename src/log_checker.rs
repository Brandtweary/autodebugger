use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use crate::config::Config;

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
        let total = self.total_count();
        let threshold = match self.configured_level {
            Level::TRACE => self.config.verbosity.trace_threshold,
            Level::DEBUG => self.config.verbosity.debug_threshold,
            Level::INFO => self.config.verbosity.info_threshold,
            Level::WARN => self.config.verbosity.warn_threshold,
            Level::ERROR => self.config.verbosity.error_threshold,
        };
        
        if total > threshold {
            Some(VerbosityWarning {
                total_count: total,
                threshold,
                configured_level: self.configured_level,
                counts: self.counts_by_level(),
            })
        } else {
            None
        }
    }
    
    /// Check verbosity and generate a formatted report if threshold exceeded
    pub fn check_and_report(&self) -> Option<String> {
        self.check_verbosity().map(|warning| {
            format!(
                "\n⚠️  Log Verbosity Warning\n\
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

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    
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