//! Lines GRBL sends back (`docs/spec/device.md`, "Protocol text").

use crate::MachineState;
use crate::grbl::BANNER;

/// A parsed status report: `<State|MPos:x,y,z|FS:f,s|WCO:x,y,z|…>`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusReport {
    /// Machine state.
    pub state: MachineState,
    /// Sub-state after `:` (`Hold:0`), if any.
    pub substate: Option<u8>,
    /// Machine position, if reported.
    pub mpos: Option<[f64; 3]>,
    /// Work position, if reported.
    pub wpos: Option<[f64; 3]>,
    /// Work coordinate offset, if included in this report.
    pub wco: Option<[f64; 3]>,
}

/// One line from GRBL.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// The oldest outstanding line was accepted.
    Ok,
    /// The oldest outstanding line was thrown away.
    Error(u16),
    /// An alarm.
    Alarm(u16),
    /// A status report.
    Status(StatusReport),
    /// The start-up banner with the version (`1.1h`), sent after every reset.
    Banner(String),
    /// A `[MSG:…]` feedback message.
    Message(String),
    /// Build options from `$I`: planner blocks and receive buffer bytes.
    Options {
        /// Planner blocks.
        blocks: Option<usize>,
        /// Receive buffer size in bytes.
        rx_buffer: Option<usize>,
    },
    /// A `$n=value` setting line.
    Setting(u16, String),
    /// Anything else (other `[…]` lines, echoes).
    Other(String),
}

impl Reply {
    /// Whether this answers a sent line (`ok` or `error:`); everything else is
    /// a push message that does not count for streaming.
    #[must_use]
    pub fn is_answer(&self) -> bool {
        matches!(self, Self::Ok | Self::Error(_))
    }
}

fn axes(text: &str) -> Option<[f64; 3]> {
    let mut out = [0.0; 3];
    let mut count = 0;
    for (slot, value) in out.iter_mut().zip(text.split(',')) {
        *slot = value.trim().parse().ok().filter(|v: &f64| v.is_finite())?;
        count += 1;
    }
    (count >= 2).then_some(out)
}

fn status(body: &str) -> Option<StatusReport> {
    let mut fields = body.split('|');
    let state_field = fields.next()?;
    let state = MachineState::parse(state_field)?;
    let substate = state_field
        .split_once(':')
        .and_then(|(_, sub)| sub.parse().ok());
    let mut report = StatusReport {
        state,
        substate,
        mpos: None,
        wpos: None,
        wco: None,
    };
    for field in fields {
        let Some((name, value)) = field.split_once(':') else {
            continue;
        };
        match name {
            "MPos" => report.mpos = axes(value),
            "WPos" => report.wpos = axes(value),
            "WCO" => report.wco = axes(value),
            _ => {}
        }
    }
    Some(report)
}

fn options(body: &str) -> Reply {
    let parts: Vec<&str> = body.split(',').collect();
    let number = |i: usize| parts.get(i).and_then(|p| p.trim().parse().ok());
    Reply::Options {
        blocks: number(1),
        rx_buffer: number(2),
    }
}

/// Parses one line (without its line ending).
#[must_use]
pub fn parse(line: &str) -> Reply {
    let line = line.trim();
    if line == "ok" {
        return Reply::Ok;
    }
    if let Some(code) = line.strip_prefix("error:").and_then(|c| c.parse().ok()) {
        return Reply::Error(code);
    }
    if let Some(code) = line.strip_prefix("ALARM:").and_then(|c| c.parse().ok()) {
        return Reply::Alarm(code);
    }
    if let Some(report) = line
        .strip_prefix('<')
        .and_then(|l| l.strip_suffix('>'))
        .and_then(status)
    {
        return Reply::Status(report);
    }
    if let Some(rest) = line.strip_prefix(BANNER) {
        let version = rest.split_whitespace().next().unwrap_or_default();
        return Reply::Banner(version.to_owned());
    }
    if let Some(body) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
        if let Some(msg) = body.strip_prefix("MSG:") {
            return Reply::Message(msg.to_owned());
        }
        if let Some(opt) = body.strip_prefix("OPT:") {
            return options(opt);
        }
    }
    if let Some((key, value)) = line.strip_prefix('$').and_then(|l| l.split_once('='))
        && let Ok(number) = key.parse()
    {
        return Reply::Setting(number, value.to_owned());
    }
    Reply::Other(line.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_and_alarms() {
        assert_eq!(parse("ok"), Reply::Ok);
        assert_eq!(parse("ok\r"), Reply::Ok);
        assert_eq!(parse("error:22"), Reply::Error(22));
        assert_eq!(parse("ALARM:3"), Reply::Alarm(3));
        assert!(parse("ok").is_answer());
        assert!(!parse("ALARM:1").is_answer());
        assert_eq!(parse("error:x"), Reply::Other("error:x".into()));
    }

    #[test]
    fn status_reports() {
        let Reply::Status(r) =
            parse("<Hold:0|MPos:10.500,2.000,0.000|FS:0,0|WCO:1.000,1.000,0.000>")
        else {
            panic!("not a status");
        };
        assert_eq!(r.state, MachineState::Hold);
        assert_eq!(r.substate, Some(0));
        assert_eq!(r.mpos, Some([10.5, 2.0, 0.0]));
        assert_eq!(r.wco, Some([1.0, 1.0, 0.0]));
        assert_eq!(r.wpos, None);
        let Reply::Status(r) = parse("<Idle|WPos:1,2,3>") else {
            panic!("not a status");
        };
        assert_eq!(
            (r.state, r.substate, r.wpos),
            (MachineState::Idle, None, Some([1.0, 2.0, 3.0]))
        );
        assert_eq!(
            parse("<Dancing|MPos:0,0,0>"),
            Reply::Other("<Dancing|MPos:0,0,0>".into())
        );
    }

    #[test]
    fn banner_messages_options_and_settings() {
        assert_eq!(
            parse("Grbl 1.1h ['$' for help]"),
            Reply::Banner("1.1h".into())
        );
        assert_eq!(
            parse("[MSG:'$H'|'$X' to unlock]"),
            Reply::Message("'$H'|'$X' to unlock".into())
        );
        assert_eq!(
            parse("[OPT:VL,15,128]"),
            Reply::Options {
                blocks: Some(15),
                rx_buffer: Some(128)
            }
        );
        assert_eq!(parse("$32=1"), Reply::Setting(32, "1".into()));
        assert_eq!(parse("$N0="), Reply::Other("$N0=".into()));
        assert_eq!(
            parse("[VER:1.1h.20190825:]"),
            Reply::Other("[VER:1.1h.20190825:]".into())
        );
    }
}
