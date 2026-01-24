use crate::types::*;
use std::collections::HashMap;
use std::time::SystemTime;

pub struct StreamParser {
    iteration: u32,
    token_usage: TokenUsage,
    warn_threshold: u32,
    rotate_threshold: u32,

    // Gutter detection
    command_failures: HashMap<String, u32>,
    file_writes: HashMap<String, Vec<SystemTime>>,
    gutter_fail_count: u32,
    gutter_thrash_count: u32,
}

impl StreamParser {
    pub fn new(
        iteration: u32,
        token_usage: TokenUsage,
        warn_threshold: u32,
        rotate_threshold: u32,
    ) -> Self {
        Self {
            iteration,
            token_usage,
            warn_threshold,
            rotate_threshold,
            command_failures: HashMap::new(),
            file_writes: HashMap::new(),
            gutter_fail_count: 3,
            gutter_thrash_count: 5,
        }
    }

    pub fn parse_activity(&mut self, kind: ActivityKind) -> (ActivityEntry, Option<Signal>) {
        // Update token usage
        match &kind {
            ActivityKind::Read { bytes, .. } => {
                self.token_usage.read += bytes;
                self.token_usage.total += bytes;
            }
            ActivityKind::Write { bytes, .. } => {
                self.token_usage.write += bytes;
                self.token_usage.total += bytes;
            }
            ActivityKind::Shell { .. } => {
                // Shell commands contribute to token usage (estimate)
                self.token_usage.shell += 100;
                self.token_usage.total += 100;
            }
            _ => {}
        }

        // Check for gutter conditions
        let mut signal = None;

        match &kind {
            ActivityKind::Shell { command, exit_code } if *exit_code != 0 => {
                // Track failed commands
                let count = self.command_failures.entry(command.clone()).or_insert(0);
                *count += 1;

                if *count >= self.gutter_fail_count {
                    signal = Some(Signal::Gutter(format!(
                        "Command failed {} times: {}",
                        count, command
                    )));
                }
            }
            ActivityKind::Write { path, .. } => {
                // Track file thrashing
                let writes = self
                    .file_writes
                    .entry(path.clone())
                    .or_insert_with(Vec::new);
                writes.push(SystemTime::now());

                // Keep only writes from last 10 minutes
                let ten_mins_ago = SystemTime::now()
                    .checked_sub(std::time::Duration::from_secs(600))
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                writes.retain(|&t| t > ten_mins_ago);

                if writes.len() >= self.gutter_thrash_count as usize {
                    signal = Some(Signal::Gutter(format!(
                        "File thrashing detected: {} ({} writes in 10min)",
                        path,
                        writes.len()
                    )));
                }
            }
            _ => {}
        }

        // Check token thresholds
        if signal.is_none() {
            if self.token_usage.total >= self.rotate_threshold {
                signal = Some(Signal::Rotate);
            } else if self.token_usage.total >= self.warn_threshold {
                signal = Some(Signal::Warn);
            }
        }

        let health = self
            .token_usage
            .health(self.warn_threshold, self.rotate_threshold);

        let entry = ActivityEntry {
            timestamp: SystemTime::now(),
            iteration: self.iteration,
            kind,
            health,
        };

        (entry, signal)
    }

    pub fn token_usage(&self) -> &TokenUsage {
        &self.token_usage
    }

    pub fn reset_tokens(&mut self) {
        self.token_usage = TokenUsage::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_tracking() {
        let mut parser = StreamParser::new(0, TokenUsage::default(), 70_000, 80_000);

        let (entry, signal) = parser.parse_activity(ActivityKind::Read {
            path: "test.rs".to_string(),
            lines: 100,
            bytes: 5000,
        });

        assert_eq!(parser.token_usage().read, 5000);
        assert_eq!(parser.token_usage().total, 5000);
        assert!(signal.is_none());
    }

    #[test]
    fn test_warn_threshold() {
        let mut parser = StreamParser::new(0, TokenUsage::default(), 70_000, 80_000);

        let (_, signal) = parser.parse_activity(ActivityKind::Read {
            path: "test.rs".to_string(),
            lines: 1000,
            bytes: 70_000,
        });

        assert!(matches!(signal, Some(Signal::Warn)));
    }

    #[test]
    fn test_rotate_threshold() {
        let mut parser = StreamParser::new(0, TokenUsage::default(), 70_000, 80_000);

        let (_, signal) = parser.parse_activity(ActivityKind::Read {
            path: "test.rs".to_string(),
            lines: 1000,
            bytes: 80_000,
        });

        assert!(matches!(signal, Some(Signal::Rotate)));
    }

    #[test]
    fn test_gutter_detection() {
        let mut parser = StreamParser::new(0, TokenUsage::default(), 70_000, 80_000);

        // Fail the same command 3 times
        for _ in 0..3 {
            let (_, signal) = parser.parse_activity(ActivityKind::Shell {
                command: "npm test".to_string(),
                exit_code: 1,
            });

            if signal.is_some() {
                assert!(matches!(signal, Some(Signal::Gutter(_))));
                break;
            }
        }
    }
}
