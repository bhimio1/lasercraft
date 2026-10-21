//! The device thread (`docs/spec/device.md`, "Threading and messages"): owns
//! the port and a [`Controller`], talks to the UI only through channels.

use std::io::{self, Read as _, Write as _};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use crate::controller::{Controller, Settings};
use crate::permissions::{LinuxFamily, linux_help};
use crate::sim::{Sim, SimConfig};
use crate::{Event, Request, SIMULATOR_PORT};

/// How often the thread wakes when nothing arrives.
const WAKE: Duration = Duration::from_millis(5);

/// The UI's end of the device thread.
#[derive(Debug)]
pub struct DeviceHandle {
    requests: Sender<Request>,
    events: Receiver<Event>,
}

impl DeviceHandle {
    /// Sends a request; `false` if the thread has gone.
    pub fn send(&self, request: Request) -> bool {
        self.requests.send(request).is_ok()
    }

    /// Every event that has arrived, without waiting.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }
}

/// A byte pipe to a controller.
trait Link: Send {
    fn write(&mut self, bytes: &[u8]) -> io::Result<()>;
    /// Reads what is available; `Ok(0)` when nothing is.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

struct SerialLink(Box<dyn serialport::SerialPort>);

impl Link for SerialLink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.0.write_all(bytes)?;
        self.0.flush()
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.read(buf) {
            Ok(0) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the port closed",
            )),
            Err(e) if e.kind() == io::ErrorKind::TimedOut => Ok(0),
            other => other,
        }
    }
}

/// The simulator on real time.
struct SimLink {
    sim: Sim,
    last: Instant,
    unread: Vec<u8>,
}

impl Link for SimLink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.sim.write(bytes);
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let now = Instant::now();
        self.sim.advance((now - self.last).as_secs_f64().min(0.25));
        self.last = now;
        self.unread.extend_from_slice(self.sim.read().as_bytes());
        let n = self.unread.len().min(buf.len());
        buf[..n].copy_from_slice(&self.unread[..n]);
        self.unread.drain(..n);
        Ok(n)
    }
}

/// Serial port names, then [`SIMULATOR_PORT`].
#[must_use]
pub fn list_ports() -> Vec<String> {
    let mut ports: Vec<String> = serialport::available_ports()
        .map(|list| list.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default();
    ports.sort();
    ports.push(SIMULATOR_PORT.to_owned());
    ports
}

fn permission_help(port: &str) -> String {
    if cfg!(target_os = "linux") {
        let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        let family = LinuxFamily::from_os_release(&release);
        let flatpak = std::path::Path::new("/.flatpak-info").exists();
        linux_help(port, family, flatpak)
    } else {
        format!("{port} could not be opened: permission denied. Close other programs that use it.")
    }
}

fn open(port: &str, baud: u32) -> Result<Box<dyn Link>, Event> {
    if port == SIMULATOR_PORT {
        return Ok(Box::new(SimLink {
            sim: Sim::new(SimConfig::default()),
            last: Instant::now(),
            unread: Vec::new(),
        }));
    }
    match serialport::new(port, baud)
        .timeout(Duration::from_millis(1))
        .open()
    {
        Ok(p) => Ok(Box::new(SerialLink(p))),
        Err(e)
            if matches!(
                e.kind(),
                serialport::ErrorKind::Io(io::ErrorKind::PermissionDenied)
            ) =>
        {
            Err(Event::PermissionDenied {
                port: port.to_owned(),
                help: permission_help(port),
            })
        }
        Err(e) => Err(Event::Disconnected {
            reason: format!("cannot open {port}: {e}"),
        }),
    }
}

struct Connection {
    link: Box<dyn Link>,
    controller: Controller,
    partial: Vec<u8>,
}

struct Worker {
    events: Sender<Event>,
    wake: Box<dyn Fn() + Send>,
    start: Instant,
    connection: Option<Connection>,
}

impl Worker {
    fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    fn emit(&self, event: Event) {
        let _ = self.events.send(event);
        (self.wake)();
    }

    fn handle(&mut self, request: Request) {
        let now = self.now();
        match (request, self.connection.as_mut()) {
            (Request::Connect { port, baud }, None) => match open(&port, baud) {
                Ok(link) => {
                    self.connection = Some(Connection {
                        link,
                        controller: Controller::new(port, Settings::default(), now),
                        partial: Vec::new(),
                    });
                }
                Err(event) => self.emit(event),
            },
            (Request::Disconnect, Some(c)) => {
                c.controller.request(Request::Disconnect, now);
                let _ = c.link.write(&c.controller.take_output());
                for event in c.controller.take_events() {
                    let _ = self.events.send(event);
                }
                self.connection = None;
                self.emit(Event::Disconnected {
                    reason: "disconnected".to_owned(),
                });
            }
            (Request::Disconnect, None) => {}
            (request, Some(c)) => c.controller.request(request, now),
            (_, None) => self.emit(Event::Rejected("not connected".to_owned())),
        }
    }

    /// Reads, ticks, writes and forwards events for the open connection.
    fn service(&mut self) {
        let now = self.now();
        let Some(c) = self.connection.as_mut() else {
            return;
        };
        let mut buf = [0u8; 1024];
        let failure = loop {
            match c.link.read(&mut buf) {
                Ok(0) => break None,
                Ok(n) => c.partial.extend_from_slice(&buf[..n]),
                Err(e) => break Some(e.to_string()),
            }
        };
        while let Some(end) = c.partial.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = c.partial.drain(..=end).collect();
            c.controller.received(&String::from_utf8_lossy(&line), now);
        }
        match failure {
            Some(reason) => c.controller.port_lost(&reason),
            None => c.controller.tick(now),
        }
        let output = c.controller.take_output();
        if !output.is_empty()
            && let Err(e) = c.link.write(&output)
        {
            c.controller.port_lost(&e.to_string());
        }
        let events = c.controller.take_events();
        let closed = c.controller.is_closed();
        if closed {
            self.connection = None;
        }
        if !events.is_empty() {
            for event in events {
                let _ = self.events.send(event);
            }
            (self.wake)();
        }
    }
}

/// Starts the device thread. `wake` is called whenever events are sent (the
/// app asks egui for a repaint). The thread ends when the handle is dropped.
pub fn spawn(wake: impl Fn() + Send + 'static) -> DeviceHandle {
    let (request_tx, request_rx) = mpsc::channel::<Request>();
    let (event_tx, event_rx) = mpsc::channel();
    let mut worker = Worker {
        events: event_tx,
        wake: Box::new(wake),
        start: Instant::now(),
        connection: None,
    };
    std::thread::Builder::new()
        .name("lasercraft-device".to_owned())
        .spawn(move || {
            loop {
                let mut batch = Vec::new();
                match request_rx.recv_timeout(WAKE) {
                    Ok(r) => batch.push(r),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        worker.handle(Request::Disconnect);
                        return;
                    }
                }
                batch.extend(request_rx.try_iter());
                // Pause, resume and stop go first (safety rule 8).
                batch.sort_by_key(|r| {
                    !matches!(r, Request::Pause | Request::Resume | Request::Stop)
                });
                for request in batch {
                    worker.handle(request);
                }
                worker.service();
            }
        })
        .expect("the OS could not start the device thread");
    DeviceHandle {
        requests: request_tx,
        events: event_rx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait_for(handle: &DeviceHandle, seconds: f64, want: impl Fn(&Event) -> bool) -> Vec<Event> {
        let start = Instant::now();
        let mut seen = Vec::new();
        while start.elapsed().as_secs_f64() < seconds {
            for e in handle.events() {
                let done = want(&e);
                seen.push(e);
                if done {
                    return seen;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out; saw {seen:?}");
    }

    #[test]
    fn the_thread_connects_to_the_simulator_and_disconnects() {
        let handle = spawn(|| {});
        assert!(handle.send(Request::Home));
        wait_for(
            &handle,
            2.0,
            |e| matches!(e, Event::Rejected(r) if r == "not connected"),
        );
        handle.send(Request::Connect {
            port: SIMULATOR_PORT.to_owned(),
            baud: 115_200,
        });
        wait_for(&handle, 5.0, |e| matches!(e, Event::Ready(_)));
        wait_for(&handle, 5.0, |e| matches!(e, Event::Status { .. }));
        handle.send(Request::Jog {
            x: 5.0,
            y: 0.0,
            feed: 6000.0,
            relative: true,
        });
        wait_for(
            &handle,
            5.0,
            |e| matches!(e, Event::Status { position: Some(p), .. } if (p[0] - 5.0).abs() < 1e-9),
        );
        handle.send(Request::Disconnect);
        wait_for(&handle, 2.0, |e| matches!(e, Event::Disconnected { .. }));
    }

    #[test]
    fn opening_a_missing_port_fails_cleanly() {
        let handle = spawn(|| {});
        handle.send(Request::Connect {
            port: "/dev/lasercraft-no-such-port".to_owned(),
            baud: 115_200,
        });
        let seen = wait_for(&handle, 2.0, |e| {
            matches!(
                e,
                Event::Disconnected { .. } | Event::PermissionDenied { .. }
            )
        });
        assert!(
            matches!(seen.last(), Some(Event::Disconnected { reason }) if reason.contains("cannot open"))
        );
    }

    #[test]
    fn ports_end_with_the_simulator() {
        assert_eq!(
            list_ports().last().map(String::as_str),
            Some(SIMULATOR_PORT)
        );
    }
}
