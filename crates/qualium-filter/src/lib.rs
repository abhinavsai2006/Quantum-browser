//! Qualium Content and Request Blocking Subsystem

pub mod engine;
pub mod rules;

pub use engine::FilterEngine;
pub use rules::{FilterAction, NetworkFilterRule, TrackerCategory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_and_tracker_blocking() {
        let engine = FilterEngine::default();

        // Should block doubleclick third-party ad
        let res1 = engine.check_url("https://googleads.g.doubleclick.net/pagead/ads?id=123", "news.com");
        assert_eq!(res1, FilterAction::Block(TrackerCategory::Advertising));

        // Should block google-analytics tracker
        let res2 = engine.check_url("https://www.google-analytics.com/analytics.js", "example.org");
        assert_eq!(res2, FilterAction::Block(TrackerCategory::Analytics));

        // Should block fingerprinting script
        let res3 = engine.check_url("https://fpjs.sh/v3/agent.js", "shop.com");
        assert_eq!(res3, FilterAction::Block(TrackerCategory::Fingerprinting));

        // Should allow legitimate first-party resource
        let res4 = engine.check_url("https://news.com/static/style.css", "news.com");
        assert_eq!(res4, FilterAction::Allow);
    }
}
