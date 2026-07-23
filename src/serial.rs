//! `to_xsd()` implementations — produce the exact lexical form for a Turtle
//! typed literal (§3.4).

use crate::types::{Date, DateTime, Duration, Time, UtcOffset};

impl UtcOffset {
    /// Render as `±HH:MM`. UTC is `+00:00` (per §4.3 example, not `Z`).
    pub fn to_xsd(&self) -> String {
        let sign = if self.total_minutes < 0 { '-' } else { '+' };
        let abs = self.total_minutes.abs();
        format!("{sign}{:02}:{:02}", abs / 60, abs % 60)
    }
}

impl Date {
    /// `YYYY-MM-DD`. Years outside 0..=9999 keep a sign and full width.
    pub fn to_xsd(&self) -> String {
        if self.year < 0 {
            format!("-{:04}-{:02}-{:02}", -self.year, self.month, self.day)
        } else {
            format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
        }
    }
}

impl Time {
    /// `HH:MM:SS[±HH:MM]`.
    pub fn to_xsd(&self) -> String {
        let mut s = format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second);
        if let Some(off) = self.offset {
            s.push_str(&off.to_xsd());
        }
        s
    }
}

impl DateTime {
    /// `YYYY-MM-DDTHH:MM:SS[±HH:MM]`.
    pub fn to_xsd(&self) -> String {
        format!("{}T{}", self.date.to_xsd(), self.time.to_xsd())
    }
}

impl Duration {
    /// `[-]P[nY][nM][nW][nD][T[nH][nM][nS]]`. All-zero renders `PT0S`.
    pub fn to_xsd(&self) -> String {
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        out.push('P');
        if self.years != 0 {
            out.push_str(&format!("{}Y", self.years));
        }
        if self.months != 0 {
            out.push_str(&format!("{}M", self.months));
        }
        if self.weeks != 0 {
            out.push_str(&format!("{}W", self.weeks));
        }
        if self.days != 0 {
            out.push_str(&format!("{}D", self.days));
        }
        let has_time = self.hours != 0 || self.minutes != 0 || self.seconds != 0;
        if has_time {
            out.push('T');
            if self.hours != 0 {
                out.push_str(&format!("{}H", self.hours));
            }
            if self.minutes != 0 {
                out.push_str(&format!("{}M", self.minutes));
            }
            if self.seconds != 0 {
                out.push_str(&format!("{}S", self.seconds));
            }
        }
        // Nothing emitted at all → a zero-length duration.
        if out.ends_with('P') {
            out.push_str("T0S");
        }
        out
    }
}
