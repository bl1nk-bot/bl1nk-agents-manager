use crate::agents::types::RateLimit;
use crate::config::RateLimitingConfig;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

pub struct RateLimitTracker {
    config: RateLimitingConfig,
    usage: HashMap<String, AgentUsage>,
}

#[derive(Debug, Clone)]
struct AgentUsage {
    requests_today: u32,
    requests_this_minute: u32,
    day_start: DateTime<Utc>,
    minute_start: DateTime<Utc>,
}

impl AgentUsage {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            requests_today: 0,
            requests_this_minute: 0,
            day_start: now,
            minute_start: now,
        }
    }

    fn reset_if_needed(&mut self) {
        let now = Utc::now();

        // Reset daily counter
        if now.signed_duration_since(self.day_start) >= Duration::days(1) {
            self.requests_today = 0;
            self.day_start = now;
        }

        // Reset minute counter
        if now.signed_duration_since(self.minute_start) >= Duration::minutes(1) {
            self.requests_this_minute = 0;
            self.minute_start = now;
        }
    }
}

impl RateLimitTracker {
    pub fn new(config: RateLimitingConfig) -> Self {
        Self {
            config,
            usage: HashMap::new(),
        }
    }

    /// Check if agent can make a request and increment counter if yes.
    /// This now takes the agent's specific rate limit configuration.
    pub async fn check_and_increment(&mut self, agent_id: &str, agent_limit: &RateLimit) -> bool {
        // ถ้าไม่ได้เปิดใช้งานการติดตาม rate limit ก็ให้ผ่านเสมอ
        if !self.config.track_usage {
            return true;
        }

        let usage = self
            .usage
            .entry(agent_id.to_string())
            .or_insert_with(AgentUsage::new);

        usage.reset_if_needed();

        let daily_limit = agent_limit.requests_per_day;
        let minute_limit = agent_limit.requests_per_minute;

        if usage.requests_today >= daily_limit {
            tracing::warn!(
                "Agent {} hit daily rate limit ({} requests)",
                agent_id,
                daily_limit
            );
            return false;
        }

        if usage.requests_this_minute >= minute_limit {
            tracing::warn!(
                "Agent {} hit per-minute rate limit ({} requests)",
                agent_id,
                minute_limit
            );
            return false;
        }

        // Increment counters
        usage.requests_today += 1;
        usage.requests_this_minute += 1;

        tracing::debug!(
            "Agent {} usage: {}/{} per day, {}/{} per minute",
            agent_id,
            usage.requests_today,
            daily_limit,
            usage.requests_this_minute,
            minute_limit
        );

        true
    }

    /// Get current usage for an agent
    pub fn get_usage(&self, agent_id: &str) -> Option<(u32, u32)> {
        self.usage
            .get(agent_id)
            .map(|usage| (usage.requests_today, usage.requests_this_minute))
    }

    /// Reset all usage counters (for testing)
    pub fn reset_all(&mut self) {
        self.usage.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting_with_specific_limits() {
        let config = RateLimitingConfig {
            strategy: "round-robin".to_string(),
            track_usage: true,
            usage_db_path: None,
        };

        let mut tracker = RateLimitTracker::new(config);

        // กำหนด limit สำหรับ agent นี้โดยเฉพาะ
        let agent_limit = RateLimit {
            requests_per_minute: 10,
            requests_per_day: 100,
        };

        // First request should succeed
        assert!(
            tracker
                .check_and_increment("test-agent", &agent_limit)
                .await
        );

        // Check usage
        let (daily, minute) = tracker.get_usage("test-agent").unwrap();
        assert_eq!(daily, 1);
        assert_eq!(minute, 1);
    }

    #[tokio::test]
    async fn test_minute_limit_is_enforced() {
        let config = RateLimitingConfig {
            strategy: "round-robin".to_string(),
            track_usage: true,
            usage_db_path: None,
        };

        let mut tracker = RateLimitTracker::new(config);

        let agent_limit = RateLimit {
            requests_per_minute: 5,
            requests_per_day: 100,
        };

        // Exhaust minute limit
        for i in 0..5 {
            assert!(
                tracker
                    .check_and_increment("test-agent", &agent_limit)
                    .await,
                "Request {} should have succeeded",
                i + 1
            );
        }

        // 6th request should fail
        assert!(
            !tracker
                .check_and_increment("test-agent", &agent_limit)
                .await,
            "The 6th request should have been rate limited"
        );
    }

    #[tokio::test]
    async fn test_tracking_disabled() {
        let config = RateLimitingConfig {
            strategy: "round-robin".to_string(),
            track_usage: false,
            usage_db_path: None,
        };

        let mut tracker = RateLimitTracker::new(config);
        let agent_limit = RateLimit {
            requests_per_minute: 1,
            requests_per_day: 1,
        };

        // ควรจะผ่านเสมอแม้ว่าจะเกิน limit
        assert!(
            tracker
                .check_and_increment("test-agent", &agent_limit)
                .await
        );
        assert!(
            tracker
                .check_and_increment("test-agent", &agent_limit)
                .await
        );
    }
}
