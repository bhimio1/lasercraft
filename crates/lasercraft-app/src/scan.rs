//! Scanning the bed with a camera on the head (`docs/spec/camera.md`,
//! "Cameras on the laser head"): move with the laser off, wait until the
//! machine is idle and the camera has fresh pictures, paint, go on.

use std::time::{Duration, Instant};

use lasercraft_geom::Point;
use lasercraft_ui::{LaserView, MachineWord};

/// Time a move may take before the scan gives up.
const MOVE_LIMIT: Duration = Duration::from_secs(60);
/// Time before the first status after a move can be trusted.
const SETTLE_STATUS: Duration = Duration::from_millis(300);
/// Time the head rests before pictures count.
const REST: Duration = Duration::from_millis(500);

/// A scan in progress.
pub struct Scan {
    /// The head camera.
    pub camera: String,
    stops: Vec<Point>,
    next: usize,
    /// Pictures to let pass after resting.
    skip: u64,
    phase: Phase,
}

enum Phase {
    Moving(Instant),
    Resting { since: Instant, from: u64 },
}

/// What the app should do now.
#[derive(Debug, PartialEq)]
pub enum Step {
    /// Nothing yet.
    Wait,
    /// Paint the camera's latest picture.
    Capture,
    /// Move the head, laser off, to this document point.
    MoveTo(Point),
    /// The scan finished.
    Done(String),
    /// The scan stopped.
    Failed(String),
}

impl Scan {
    /// A scan over `stops` (document mm); the caller sends the first move.
    pub fn new(camera: String, stops: Vec<Point>, skip: u32, now: Instant) -> Self {
        Self {
            camera,
            stops,
            next: 0,
            skip: u64::from(skip),
            phase: Phase::Moving(now),
        }
    }

    /// The first stop.
    pub fn first(&self) -> Option<Point> {
        self.stops.first().copied()
    }

    /// Stops painted so far (counting the one under way) and in all.
    pub fn progress(&self) -> (usize, usize) {
        (self.next + 1, self.stops.len())
    }

    /// What to do, given the laser, the head (document mm) and how many
    /// pictures the camera has delivered.
    pub fn step(
        &mut self,
        laser: &LaserView,
        head: Option<Point>,
        pictures: u64,
        now: Instant,
    ) -> Step {
        if laser.port.is_none() {
            return Step::Failed("Scan stopped: the laser disconnected.".to_owned());
        }
        if laser.alarm.is_some() || laser.word == MachineWord::Alarm {
            return Step::Failed("Scan stopped: the laser reports an alarm.".to_owned());
        }
        let Some(target) = self.stops.get(self.next).copied() else {
            return Step::Done(self.done());
        };
        match self.phase {
            Phase::Moving(sent) => {
                if now.duration_since(sent) > MOVE_LIMIT {
                    return Step::Failed(
                        "Scan stopped: the head did not reach its next stop.".to_owned(),
                    );
                }
                let there = head.is_some_and(|h| h.distance(target) < 0.2);
                if now.duration_since(sent) >= SETTLE_STATUS
                    && laser.word == MachineWord::Ready
                    && there
                {
                    self.phase = Phase::Resting {
                        since: now,
                        from: pictures,
                    };
                }
                Step::Wait
            }
            Phase::Resting { since, from } => {
                if now.duration_since(since) >= REST && pictures > from + self.skip {
                    Step::Capture
                } else {
                    Step::Wait
                }
            }
        }
    }

    /// The picture was painted: the next stop, or done.
    pub fn captured(&mut self, now: Instant) -> Step {
        self.next += 1;
        match self.stops.get(self.next) {
            Some(p) => {
                self.phase = Phase::Moving(now);
                Step::MoveTo(*p)
            }
            None => Step::Done(self.done()),
        }
    }

    fn done(&self) -> String {
        format!(
            "Scanned the bed with \"{}\": {} pictures.",
            self.camera,
            self.stops.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scan_waits_for_the_head_rest_and_fresh_pictures() {
        let t0 = Instant::now();
        let stops = vec![Point::new(10.0, 10.0), Point::new(50.0, 10.0)];
        let mut scan = Scan::new("Nozzle".into(), stops, 2, t0);
        let mut laser = LaserView {
            port: Some("simulator".into()),
            word: MachineWord::Jogging,
            ..LaserView::default()
        };
        let at = |s: f64| t0 + Duration::from_secs_f64(s);
        let head = Some(Point::new(10.0, 10.0));
        assert_eq!(
            scan.step(&laser, head, 5, at(1.0)),
            Step::Wait,
            "still moving"
        );
        laser.word = MachineWord::Ready;
        assert_eq!(
            scan.step(&laser, Some(Point::ZERO), 5, at(1.0)),
            Step::Wait,
            "not there"
        );
        assert_eq!(
            scan.step(&laser, head, 5, at(1.0)),
            Step::Wait,
            "resting starts"
        );
        assert_eq!(
            scan.step(&laser, head, 9, at(1.2)),
            Step::Wait,
            "rests half a second"
        );
        assert_eq!(
            scan.step(&laser, head, 7, at(1.6)),
            Step::Wait,
            "skips two pictures"
        );
        assert_eq!(scan.step(&laser, head, 8, at(1.6)), Step::Capture);
        assert_eq!(scan.captured(at(1.7)), Step::MoveTo(Point::new(50.0, 10.0)));
        assert_eq!(scan.progress(), (2, 2));
        laser.alarm = Some("ALARM:1".into());
        assert!(matches!(
            scan.step(&laser, head, 9, at(2.0)),
            Step::Failed(_)
        ));
        laser.alarm = None;
        assert!(matches!(
            scan.step(&laser, head, 9, at(99.0)),
            Step::Failed(_)
        ));
        assert!(matches!(scan.captured(at(99.0)), Step::Done(_)));
    }
}
