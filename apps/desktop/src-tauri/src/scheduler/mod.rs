//! Queue and schedule orchestration.

use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDecision {
    Waiting,
    Ready,
    Expired,
}

/// Evaluates an optional local-time schedule window.
/// Empty boundaries mean unbounded on that side.
pub fn evaluate_window(
    now: DateTime<Local>,
    start_at: Option<&str>,
    stop_at: Option<&str>,
) -> Result<ScheduleDecision, String> {
    let start = start_at
        .filter(|value| !value.trim().is_empty())
        .map(parse_local)
        .transpose()?;
    let stop = stop_at
        .filter(|value| !value.trim().is_empty())
        .map(parse_local)
        .transpose()?;

    if let Some(start) = start {
        if now < start {
            return Ok(ScheduleDecision::Waiting);
        }
    }
    if let Some(stop) = stop {
        if now >= stop {
            return Ok(ScheduleDecision::Expired);
        }
    }
    Ok(ScheduleDecision::Ready)
}

fn parse_local(value: &str) -> Result<DateTime<Local>, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Local))
        .map_err(|_| format!("Invalid schedule timestamp: {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};

    #[test]
    fn schedule_window_decisions_are_deterministic() {
        let now = Local::now();
        let before = (now + Duration::minutes(5)).to_rfc3339();
        let after = (now - Duration::minutes(5)).to_rfc3339();
        assert_eq!(
            evaluate_window(now, Some(&before), None).unwrap(),
            ScheduleDecision::Waiting
        );
        assert_eq!(
            evaluate_window(now, Some(&after), Some(&before)).unwrap(),
            ScheduleDecision::Ready
        );
        assert_eq!(
            evaluate_window(now, None, Some(&after)).unwrap(),
            ScheduleDecision::Expired
        );
    }
}
