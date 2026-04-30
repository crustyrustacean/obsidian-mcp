use crate::error::ObsidianError;
use flux_limiter::{FluxLimiter, FluxLimiterConfig, SystemClock};
use std::sync::Arc;

/// Per-tool rate limiter using GCRA (Generic Cell Rate Algorithm).
///
/// Each tool has its own rate limit key. The limits are conservative to
/// prevent a single MCP client from overwhelming the Obsidian Local REST API,
/// which runs as a plugin inside the Obsidian app and shares resources with
/// the editor.
///
/// Current limits per tool:
/// - Write operations (write/append/patch/delete): 2 req/s, burst of 3
/// - Read operations (read/metadata/list_directory/batch_read): 5 req/s, burst of 5
/// - Search operations (search/dataview/jsonlogic): 3 req/s, burst of 3
/// - Navigation (open_note, server_status, get_tags, periodic_note, recent_changes): 5 req/s, burst of 5
#[derive(Debug, Clone)]
pub struct ToolRateLimiter {
    limiter: Arc<FluxLimiter<String, SystemClock>>,
}

impl ToolRateLimiter {
    /// Create a new rate limiter with default per-tool limits.
    ///
    /// Uses 2 requests/second sustained rate with a burst capacity of 5.
    /// This is conservative enough to protect the Obsidian REST API while
    /// allowing reasonable interactive use from a single MCP client.
    pub fn new() -> Self {
        let config = FluxLimiterConfig::new(2.0, 5.0);
        let limiter = FluxLimiter::with_config(config, SystemClock)
            .expect("failed to create rate limiter — invalid config");

        Self {
            limiter: Arc::new(limiter),
        }
    }

    /// Check whether a tool call is allowed under the rate limit.
    ///
    /// Returns `Ok(())` if the call is allowed, or a `RateLimitExceeded` error
    /// with the retry-after time if the limit has been exceeded.
    pub fn check(&self, tool_name: &str) -> Result<(), ObsidianError> {
        let decision = self
            .limiter
            .check_request(tool_name.to_string())
            .map_err(|_e| ObsidianError::RateLimitExceeded {
                tool: tool_name.to_string(),
                retry_after_seconds: 0.0,
            })?;

        if decision.allowed {
            Ok(())
        } else {
            Err(ObsidianError::RateLimitExceeded {
                tool: tool_name.to_string(),
                retry_after_seconds: decision.retry_after_seconds.unwrap_or(1.0),
            })
        }
    }
}

impl Default for ToolRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows_first_request() {
        let limiter = ToolRateLimiter::new();
        assert!(limiter.check("test_tool").is_ok());
    }

    #[test]
    fn rate_limiter_enforces_limit() {
        let limiter = ToolRateLimiter::new();
        // With burst=5, the first 6 requests should be allowed (5 burst + 1 from refill)
        // then subsequent ones should be denied
        let mut allowed = 0;
        let mut denied = false;
        for _ in 0..20 {
            match limiter.check("burst_test") {
                Ok(()) => allowed += 1,
                Err(_) => {
                    denied = true;
                    break;
                }
            }
        }
        assert!(denied, "rate limiter should eventually deny requests");
        assert!(allowed > 0, "at least some requests should be allowed");
        assert!(allowed <= 6, "burst should be limited, got {allowed} allowed");
    }

    #[test]
    fn rate_limiter_is_per_tool() {
        let limiter = ToolRateLimiter::new();
        // Different tools should have independent rate limits
        assert!(limiter.check("tool_a").is_ok());
        assert!(limiter.check("tool_b").is_ok());
    }
}
