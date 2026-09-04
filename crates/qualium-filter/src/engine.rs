//! High-Performance Ad and Tracker Content Filter Engine

use crate::rules::{FilterAction, NetworkFilterRule, TrackerCategory};
use std::collections::HashMap;
use url::Url;

pub struct FilterEngine {
    rules_by_domain: HashMap<String, Vec<NetworkFilterRule>>,
    total_rules_loaded: usize,
}

impl FilterEngine {
    pub fn new() -> Self {
        Self {
            rules_by_domain: HashMap::new(),
            total_rules_loaded: 0,
        }
    }

    /// Load default privacy filter lists (EasyList, EasyPrivacy, Tracking Protection)
    pub fn load_defaults(&mut self) {
        let sample_rules = [
            // Advertising
            ("||doubleclick.net^", TrackerCategory::Advertising),
            ("||googleads.g.doubleclick.net^", TrackerCategory::Advertising),
            ("||adnxs.com^", TrackerCategory::Advertising),
            ("||criteo.com^", TrackerCategory::Advertising),
            ("||adservice.google.com^", TrackerCategory::Advertising),
            ("||pagead2.googlesyndication.com^", TrackerCategory::Advertising),
            ("||amazon-adsystem.com^", TrackerCategory::Advertising),
            ("||taboola.com^", TrackerCategory::Advertising),
            ("||outbrain.com^", TrackerCategory::Advertising),
            // Analytics
            ("||google-analytics.com^", TrackerCategory::Analytics),
            ("||analytics.google.com^", TrackerCategory::Analytics),
            ("||hotjar.com^", TrackerCategory::Analytics),
            ("||segment.io^", TrackerCategory::Analytics),
            ("||mixpanel.com^", TrackerCategory::Analytics),
            ("||clarity.ms^", TrackerCategory::Analytics),
            // Social Trackers
            ("||connect.facebook.net^", TrackerCategory::SocialTracker),
            ("||pixel.facebook.com^", TrackerCategory::SocialTracker),
            ("||static.ads-twitter.com^", TrackerCategory::SocialTracker),
            ("||snap.licdn.com^", TrackerCategory::SocialTracker),
            ("||tiktok.com/api/v1/pixel^", TrackerCategory::SocialTracker),
            // Fingerprinting
            ("||fpjs.sh^", TrackerCategory::Fingerprinting),
            ("||fingerprintjs.com^", TrackerCategory::Fingerprinting),
            ("||threatmetrix.com^", TrackerCategory::Fingerprinting),
            ("||augur.io^", TrackerCategory::Fingerprinting),
            // Cryptomining & Malicious
            ("||coinhive.com^", TrackerCategory::Cryptomining),
            ("||coin-hive.com^", TrackerCategory::Cryptomining),
            ("||malware-delivery-cdn.net^", TrackerCategory::Malicious),
        ];

        for (pattern, cat) in sample_rules {
            if let Some(rule) = NetworkFilterRule::parse_abp_line(pattern, cat) {
                self.add_rule(rule);
            }
        }
    }

    pub fn add_rule(&mut self, rule: NetworkFilterRule) {
        self.rules_by_domain
            .entry(rule.domain_suffix.clone())
            .or_default()
            .push(rule);
        self.total_rules_loaded += 1;
    }

    pub fn rule_count(&self) -> usize {
        self.total_rules_loaded
    }

    /// Evaluates whether a network request URL should be Allowed or Blocked
    pub fn check_url(&self, target_url_str: &str, first_party_domain: &str) -> FilterAction {
        let Ok(parsed_url) = Url::parse(target_url_str) else {
            return FilterAction::Allow;
        };

        let Some(host) = parsed_url.host_str() else {
            return FilterAction::Allow;
        };

        let host = host.to_lowercase();

        // 1. Check exact match or domain suffix matches
        let parts: Vec<&str> = host.split('.').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let candidate_domain = parts[i..].join(".");
            if let Some(rules) = self.rules_by_domain.get(&candidate_domain) {
                for rule in rules {
                    // Check exception
                    if rule.is_exception {
                        return FilterAction::Allow;
                    }

                    // Check path substring if specified
                    if let Some(ref path_sub) = rule.path_substring {
                        if !parsed_url.path().contains(path_sub) {
                            continue;
                        }
                    }

                    // If first-party matches domain, allow if compatibility required, else block third-party tracker
                    if !first_party_domain.is_empty() && host.ends_with(first_party_domain) {
                        // Same-party
                        return FilterAction::Allow;
                    }

                    return FilterAction::Block(rule.category);
                }
            }
        }

        FilterAction::Allow
    }
}

impl Default for FilterEngine {
    fn default() -> Self {
        let mut engine = Self::new();
        engine.load_defaults();
        engine
    }
}
