//! Analysis engine that runs rules and manages insights

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::collector::MemorySnapshot;
use crate::history::HistoryBuffer;

use super::insights::Insight;
use super::rules::{
    CacheInfoRule, FragmentationDetector, MemoryHogDetector, MemoryLeakDetector,
    OomRiskDetector, Rule, SuddenSpikeDetector, SwapPressureDetector,
};

/// The main analyzer that runs all rules
pub struct Analyzer {
    /// Active rules
    rules: Vec<Box<dyn Rule>>,
    /// Current active insights (deduplicated by ID)
    active_insights: HashMap<String, Insight>,
    /// Cooldown for re-triggering the same insight
    cooldown: Duration,
    /// Last trigger time for each insight
    last_triggered: HashMap<String, Instant>,
    /// Maximum insights to keep
    max_insights: usize,
}

impl Analyzer {
    /// Create a new analyzer with default rules
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(MemoryLeakDetector::default()),
                Box::new(MemoryHogDetector::default()),
                Box::new(SuddenSpikeDetector::default()),
                Box::new(OomRiskDetector::default()),
                Box::new(SwapPressureDetector::default()),
                Box::new(FragmentationDetector::default()),
                Box::new(CacheInfoRule),
            ],
            active_insights: HashMap::new(),
            cooldown: Duration::from_secs(60), // Don't re-trigger same insight for 1 min
            last_triggered: HashMap::new(),
            max_insights: 10,
        }
    }

    /// Add a custom rule
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Set cooldown duration
    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// Analyze a snapshot and update insights
    pub fn analyze(&mut self, snapshot: &MemorySnapshot, history: &HistoryBuffer) {
        let now = Instant::now();

        for rule in &self.rules {
            if let Some(insight) = rule.evaluate(snapshot, history) {
                // Check cooldown
                if let Some(last) = self.last_triggered.get(&insight.id) {
                    if now.duration_since(*last) < self.cooldown {
                        continue; // Still in cooldown
                    }
                }

                // Add or update insight
                self.last_triggered.insert(insight.id.clone(), now);
                self.active_insights.insert(insight.id.clone(), insight);
            }
        }

        // Prune old insights (keep only the most recent)
        if self.active_insights.len() > self.max_insights {
            let mut insights: Vec<_> = self.active_insights.drain().collect();
            insights.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
            insights.truncate(self.max_insights);
            self.active_insights = insights.into_iter().collect();
        }
    }

    /// Get all active insights, sorted by severity (critical first)
    pub fn insights(&self) -> Vec<&Insight> {
        let mut insights: Vec<_> = self.active_insights.values().collect();
        insights.sort_by(|a, b| b.severity.cmp(&a.severity));
        insights
    }

    /// Get insights for a specific process
    pub fn insights_for_process(&self, pid: i32) -> Vec<&Insight> {
        self.active_insights
            .values()
            .filter(|i| i.pid == Some(pid))
            .collect()
    }

    /// Acknowledge an insight (mark as seen)
    pub fn acknowledge(&mut self, id: &str) {
        if let Some(insight) = self.active_insights.get_mut(id) {
            insight.acknowledged = true;
        }
    }

    /// Dismiss an insight
    pub fn dismiss(&mut self, id: &str) {
        self.active_insights.remove(id);
    }

    /// Clear all insights
    pub fn clear(&mut self) {
        self.active_insights.clear();
    }

    /// Get count of unacknowledged insights by severity
    pub fn unacknowledged_counts(&self) -> (usize, usize, usize) {
        let mut critical = 0;
        let mut warning = 0;
        let mut info = 0;

        for insight in self.active_insights.values() {
            if insight.acknowledged {
                continue;
            }
            match insight.severity {
                super::Severity::Critical => critical += 1,
                super::Severity::Warning => warning += 1,
                super::Severity::Info => info += 1,
            }
        }

        (critical, warning, info)
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
