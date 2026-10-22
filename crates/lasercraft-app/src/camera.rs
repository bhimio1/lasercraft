//! The app's cameras (`docs/spec/camera.md`, "Sources"): a thread per
//! camera that delivers pictures (USB through the operating system's
//! camera interface, network over HTTP, or the simulated camera), the
//! overlay built from them, and the [`CameraView`] the shell draws.

use std::collections::BTreeMap;
use std::io::Read;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use lasercraft_camera::lens::{Lens, Model};
use lasercraft_camera::overlay::Overlay;
use lasercraft_camera::synth::{Pose, Renderer};
use lasercraft_camera::{Picture, controls, sheet};
use lasercraft_geom::{Point, Rect};
use lasercraft_project::WorkArea;
use lasercraft_ui::{CameraLive, CameraStatus, CameraView, LensCapture};
use lasercraft_workshop::{Camera, Source};

/// Seconds between retries of a failed source.
const RETRY: Duration = Duration::from_secs(5);

enum Control {
    Stop,
    Rate(f64),
    /// The stored picture settings changed.
    Settings(BTreeMap<String, f64>),
}

enum Message {
    Picture(Picture),
    Failed(String),
    /// The controls the source offers, with their current values.
    Controls(Vec<controls::Control>),
    /// Something to tell the user once.
    Notice(String),
}

struct Worker {
    source: Source,
    control: Sender<Control>,
    messages: Receiver<Message>,
    status: CameraStatus,
    latest: Option<Arc<Picture>>,
    revision: u64,
    settings: BTreeMap<String, f64>,
    offered: Vec<controls::Control>,
}

/// What the simulated camera sees: the bed, everything burned on it, and
/// the calibration sheet when the lens wizard asks for it.
#[derive(Clone)]
pub struct SimScene {
    area: WorkArea,
    /// Burned bed pixels at [`SIM_MM`] per pixel.
    burned: Vec<bool>,
    /// The sheet's top-left dot on the bed and its turn, while shown.
    pub sheet: Option<(Point, f64)>,
}

/// Simulated bed resolution, mm per pixel.
const SIM_MM: f64 = 0.25;

impl SimScene {
    fn new(area: WorkArea) -> Self {
        let (w, h) = Self::size(&area);
        Self {
            area,
            burned: vec![false; w * h],
            sheet: None,
        }
    }

    fn size(area: &WorkArea) -> (usize, usize) {
        (
            (area.width_mm / SIM_MM).ceil() as usize + 1,
            (area.height_mm / SIM_MM).ceil() as usize + 1,
        )
    }

    /// Marks a burned segment (document mm), 0.3 mm wide.
    fn burn(&mut self, a: Point, b: Point) {
        let (w, h) = Self::size(&self.area);
        let steps = (a.distance(b) / (SIM_MM / 2.0)).ceil().max(1.0) as usize;
        for k in 0..=steps {
            let t = k as f64 / steps as f64;
            let p = Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
            let (x, y) = ((p.x / SIM_MM).round(), (p.y / SIM_MM).round());
            for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)] {
                let (x, y) = (x + dx, y + dy);
                if x >= 0.0 && y >= 0.0 && (x as usize) < w && (y as usize) < h {
                    self.burned[y as usize * w + x as usize] = true;
                }
            }
        }
    }

    /// Reads a streamed program (`G0`/`G1`, `M3`/`M4`/`M5`, `S`, `G90`/`G91`)
    /// and burns what the laser would.
    pub fn burn_gcode(&mut self, gcode: &str) {
        let (mut x, mut y, mut s, mut on, mut absolute) = (0.0, 0.0, 0.0, false, true);
        for line in gcode.lines() {
            let line = line.split('(').next().unwrap_or_default();
            let mut motion = None;
            let (mut nx, mut ny) = (None, None);
            for word in line.split_whitespace() {
                let (letter, value) = word.split_at(1);
                let v: f64 = value.parse().unwrap_or(f64::NAN);
                match (letter, v) {
                    ("G", g) if g == 0.0 || g == 1.0 => motion = Some(g == 1.0),
                    ("G", 90.0) => absolute = true,
                    ("G", 91.0) => absolute = false,
                    ("M", m) if m == 3.0 || m == 4.0 => on = true,
                    ("M", 5.0) => on = false,
                    ("S", v) => s = v,
                    ("X", v) => nx = Some(v),
                    ("Y", v) => ny = Some(v),
                    _ => {}
                }
            }
            let to_x = nx.map_or(x, |v| if absolute { v } else { x + v });
            let to_y = ny.map_or(y, |v| if absolute { v } else { y + v });
            if motion == Some(true) && on && s > 0.0 {
                let (a, b) = (
                    self.area.to_document(Point::new(x, y)),
                    self.area.to_document(Point::new(to_x, to_y)),
                );
                self.burn(a, b);
            }
            (x, y) = (to_x, to_y);
        }
    }

    fn color(&self, p: Point) -> Option<[u8; 3]> {
        if let Some((at, angle)) = self.sheet {
            let d = p - at;
            let (s, c) = angle.sin_cos();
            if let Some(dark) = sheet::dark_at(Point::new(c * d.x + s * d.y, -s * d.x + c * d.y)) {
                return Some(if dark { [25, 25, 25] } else { [240, 240, 236] });
            }
        }
        if p.x < 0.0 || p.y < 0.0 || p.x > self.area.width_mm || p.y > self.area.height_mm {
            return None;
        }
        let (w, _) = Self::size(&self.area);
        let i = (p.y / SIM_MM).round() as usize * w + (p.x / SIM_MM).round() as usize;
        if self.burned.get(i).copied().unwrap_or(false) {
            return Some([45, 32, 20]);
        }
        // A light board with a faint 10 mm grain.
        let grain = if (p.x / 10.0).floor() as i64 % 2 == 0 {
            0
        } else {
            8
        };
        Some([222 - grain, 200 - grain, 160 - grain])
    }
}

/// The simulated camera's own lens and where it hangs over a bed.
#[must_use]
pub fn simulated_camera(area: &WorkArea) -> (Lens, Pose) {
    let mut lens = Lens::ideal(Model::Standard, 640, 480, 430.0);
    lens.k = vec![-0.3, 0.1];
    lens.cx = 322.0;
    lens.cy = 238.0;
    // High enough that the whole bed fits in the picture.
    let height = (lens.fx * area.width_mm / (2.0 * 0.8 * 320.0))
        .max(lens.fy * area.height_mm / (2.0 * 0.8 * 240.0));
    (
        lens,
        Pose::overhead(
            area.width_mm / 2.0,
            area.height_mm / 2.0,
            height,
            [0.01, -0.012, 0.015],
        ),
    )
}

/// Where the lens wizard's five pictures want the sheet: the four corners
/// of the simulated camera's view and its center (top-left dot, turn).
#[must_use]
pub fn sheet_positions(area: &WorkArea) -> [(Point, f64); 5] {
    let (lens, pose) = simulated_camera(area);
    let corner = |u: f64, v: f64| {
        pose.hit(lens.unproject(Point::new(u, v)))
            .unwrap_or_default()
    };
    let (tl, br) = (corner(40.0, 40.0), corner(600.0, 440.0));
    let (dots_w, dots_h) = (
        f64::from(sheet::COLUMNS - 1) * sheet::PITCH,
        f64::from(sheet::ROWS - 1) * sheet::PITCH,
    );
    let c = Point::new((tl.x + br.x - dots_w) / 2.0, (tl.y + br.y - dots_h) / 2.0);
    [
        (tl + Point::new(10.0, 10.0), 0.05),
        (Point::new(br.x - dots_w - 10.0, tl.y + 10.0), -0.05),
        (Point::new(br.x - dots_w - 10.0, br.y - dots_h - 10.0), 0.03),
        (Point::new(tl.x + 10.0, br.y - dots_h - 10.0), -0.03),
        (c, 0.1),
    ]
}

/// Applies the stored settings and reports the controls with their values.
fn apply_settings(grabber: &mut dyn Grab, settings: &BTreeMap<String, f64>, tx: &Sender<Message>) {
    for notice in grabber.apply(settings) {
        let _ = tx.send(Message::Notice(notice));
    }
    let _ = tx.send(Message::Controls(grabber.controls()));
}

fn spawn(
    camera: &Camera,
    rate: f64,
    scene: &Arc<Mutex<SimScene>>,
    repaint: &egui::Context,
) -> (Sender<Control>, Receiver<Message>) {
    let (ctl_tx, ctl_rx) = channel();
    let (msg_tx, msg_rx) = channel();
    let source = camera.source.clone();
    let skip = camera.skip;
    let mut settings = camera.controls.clone();
    let scene = Arc::clone(scene);
    let ctx = repaint.clone();
    thread::spawn(move || {
        let mut grabber: Option<Box<dyn Grab>> = None;
        let mut rate = rate;
        let mut skipped = 0;
        let mut failed_at: Option<Instant> = None;
        loop {
            match ctl_rx.try_recv() {
                Ok(Control::Stop) | Err(TryRecvError::Disconnected) => return,
                Ok(Control::Rate(r)) => rate = r,
                Ok(Control::Settings(s)) => {
                    // A removed setting goes back to the camera's default.
                    let mut wanted = s.clone();
                    if let Some(g) = grabber.as_mut() {
                        for c in g.controls() {
                            if settings.contains_key(c.key) && !s.contains_key(c.key) {
                                wanted.insert(c.key.to_owned(), c.default);
                            }
                        }
                        apply_settings(g.as_mut(), &wanted, &msg_tx);
                        ctx.request_repaint();
                    }
                    settings = s;
                }
                Err(TryRecvError::Empty) => {}
            }
            if failed_at.is_some_and(|t| t.elapsed() < RETRY) {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            if grabber.is_none() {
                match open(&source, &scene) {
                    Ok(mut g) => {
                        apply_settings(g.as_mut(), &settings, &msg_tx);
                        grabber = Some(g);
                        skipped = 0;
                    }
                    Err(e) => {
                        failed_at = Some(Instant::now());
                        let _ = msg_tx.send(Message::Failed(e));
                        ctx.request_repaint();
                        continue;
                    }
                }
            }
            let started = Instant::now();
            let picture = grabber.as_mut().map(|g| g.grab());
            match picture {
                Some(Ok(p)) if skipped >= skip => {
                    failed_at = None;
                    if msg_tx.send(Message::Picture(p)).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
                Some(Ok(_)) => skipped += 1,
                Some(Err(e)) => {
                    grabber = None;
                    failed_at = Some(Instant::now());
                    let _ = msg_tx.send(Message::Failed(e));
                    ctx.request_repaint();
                    continue;
                }
                None => {}
            }
            let period = Duration::from_secs_f64(1.0 / rate.clamp(0.1, 30.0));
            if let Some(rest) = period.checked_sub(started.elapsed()) {
                thread::sleep(rest);
            }
        }
    });
    (ctl_tx, msg_rx)
}

/// A picture provider on its worker thread.
trait Grab {
    fn grab(&mut self) -> Result<Picture, String>;

    /// The controls the source offers, current values included.
    fn controls(&mut self) -> Vec<controls::Control> {
        Vec::new()
    }

    /// Applies stored settings to the controls it offers; returns what the
    /// source refused, in words for the user.
    fn apply(&mut self, _settings: &BTreeMap<String, f64>) -> Vec<String> {
        Vec::new()
    }
}

fn open(source: &Source, scene: &Arc<Mutex<SimScene>>) -> Result<Box<dyn Grab>, String> {
    match source {
        Source::Simulated => Ok(Box::new(Simulated::new(Arc::clone(scene)))),
        Source::Network { url, .. } => Ok(Box::new(Network::new(url))),
        Source::Usb { index, format, .. } => usb::open(*index, format),
    }
}

struct Simulated {
    scene: Arc<Mutex<SimScene>>,
    renderer: Option<(WorkArea, Renderer)>,
    brightness: f64,
    contrast: f64,
}

impl Simulated {
    fn new(scene: Arc<Mutex<SimScene>>) -> Self {
        Self {
            scene,
            renderer: None,
            brightness: 0.0,
            contrast: 0.0,
        }
    }
}

impl Grab for Simulated {
    fn grab(&mut self) -> Result<Picture, String> {
        let scene = self
            .scene
            .lock()
            .map_err(|_| "the simulated camera stopped".to_owned())?
            .clone();
        if self.renderer.as_ref().is_none_or(|(a, _)| *a != scene.area) {
            let (lens, pose) = simulated_camera(&scene.area);
            self.renderer = Some((scene.area, Renderer::new(&lens, &pose)));
        }
        let (_, renderer) = self.renderer.as_ref().ok_or("no renderer")?;
        let picture = renderer.render([70, 70, 74], |p| scene.color(p));
        Ok(controls::adjust(&picture, self.brightness, self.contrast))
    }

    fn controls(&mut self) -> Vec<controls::Control> {
        controls::simulated(self.brightness, self.contrast)
    }

    fn apply(&mut self, settings: &BTreeMap<String, f64>) -> Vec<String> {
        for c in controls::simulated(0.0, 0.0) {
            let v = c.fit(settings.get(c.key).copied().unwrap_or(c.default));
            match c.key {
                "brightness" => self.brightness = v,
                _ => self.contrast = v,
            }
        }
        Vec::new()
    }
}

/// A network camera: one JPEG per request, or a motion-JPEG stream.
struct Network {
    url: String,
    agent: ureq::Agent,
    stream: Option<Box<dyn Read + Send>>,
}

impl Network {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(10))
                .build(),
            stream: None,
        }
    }
}

/// The next JPEG in a byte stream: from `FF D8` to `FF D9`.
fn next_jpeg(reader: &mut dyn Read) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut byte = [0u8; 1];
    let mut prev = 0u8;
    let mut started = false;
    loop {
        reader
            .read_exact(&mut byte)
            .map_err(|e| format!("the picture stream ended: {e}"))?;
        let b = byte[0];
        if !started {
            if prev == 0xFF && b == 0xD8 {
                started = true;
                out.extend([0xFF, 0xD8]);
            }
        } else {
            out.push(b);
            if prev == 0xFF && b == 0xD9 {
                return Ok(out);
            }
            if out.len() > 32 << 20 {
                return Err("a picture in the stream is too large".to_owned());
            }
        }
        prev = b;
    }
}

impl Grab for Network {
    fn grab(&mut self) -> Result<Picture, String> {
        if let Some(stream) = &mut self.stream {
            let bytes = next_jpeg(stream.as_mut())?;
            return Picture::decode(&bytes).map_err(|e| e.0);
        }
        let response = self
            .agent
            .get(&self.url)
            .call()
            .map_err(|e| format!("cannot reach {}: {e}", self.url))?;
        if response.content_type().starts_with("multipart/") {
            let mut reader: Box<dyn Read + Send> = Box::new(response.into_reader());
            let bytes = next_jpeg(reader.as_mut())?;
            self.stream = Some(reader);
            return Picture::decode(&bytes).map_err(|e| e.0);
        }
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(32 << 20)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("cannot read the picture: {e}"))?;
        Picture::decode(&bytes).map_err(|e| e.0)
    }
}

mod usb {
    //! USB cameras through `nokhwa` (Video4Linux2, Media Foundation,
    //! AVFoundation).

    use std::collections::BTreeMap;

    use lasercraft_camera::Picture;
    use lasercraft_camera::controls::{self, Kind};
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{
        CameraControl, CameraIndex, ControlValueDescription, ControlValueSetter, FrameFormat,
        KnownCameraControl, KnownCameraControlFlag, RequestedFormat, RequestedFormatType,
        Resolution,
    };

    use super::Grab;

    struct Usb {
        camera: nokhwa::Camera,
    }

    pub(super) fn open(index: u32, format: &str) -> Result<Box<dyn Grab>, String> {
        let wanted = format
            .split_once('x')
            .and_then(|(w, h)| Some(Resolution::new(w.parse().ok()?, h.parse().ok()?)));
        let request = match wanted {
            Some(r) => RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(r)),
            None => {
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution)
            }
        };
        let mut camera = nokhwa::Camera::new(CameraIndex::Index(index), request)
            .map_err(|e| format!("camera {index}: {e}"))?;
        camera
            .open_stream()
            .map_err(|e| format!("camera {index}: {e}"))?;
        Ok(Box::new(Usb { camera }))
    }

    /// Our key of a control nokhwa knows.
    fn key(control: KnownCameraControl) -> Option<&'static str> {
        use KnownCameraControl as K;
        Some(match control {
            K::Brightness => "brightness",
            K::Contrast => "contrast",
            K::Hue => "hue",
            K::Saturation => "saturation",
            K::Sharpness => "sharpness",
            K::Gamma => "gamma",
            K::WhiteBalance => "white_balance",
            K::BacklightComp => "backlight",
            K::Gain => "gain",
            K::Pan => "pan",
            K::Tilt => "tilt",
            K::Zoom => "zoom",
            K::Exposure => "exposure",
            K::Iris => "iris",
            K::Focus => "focus",
            K::Other(_) => return None,
        })
    }

    /// A control the camera reports, as ours; `None` when LaserCraft does
    /// not offer it (unknown, read-only, disabled or not a number).
    fn offered(c: &CameraControl) -> Option<(controls::Control, bool)> {
        if c.flag().iter().any(|f| {
            matches!(
                f,
                KnownCameraControlFlag::ReadOnly | KnownCameraControlFlag::Disabled
            )
        }) {
            return None;
        }
        let key = key(c.control())?;
        let (kind, default, value, float) = match c.description() {
            ControlValueDescription::IntegerRange {
                min,
                max,
                value,
                step,
                default,
            } => (
                Kind::Range {
                    min: *min as f64,
                    max: *max as f64,
                    step: *step as f64,
                },
                *default as f64,
                *value as f64,
                false,
            ),
            ControlValueDescription::FloatRange {
                min,
                max,
                value,
                step,
                default,
            } => (
                Kind::Range {
                    min: *min,
                    max: *max,
                    step: *step,
                },
                *default,
                *value,
                true,
            ),
            ControlValueDescription::Boolean { value, default } => (
                Kind::Switch,
                f64::from(u8::from(*default)),
                f64::from(u8::from(*value)),
                false,
            ),
            _ => return None,
        };
        Some((
            controls::Control {
                key,
                kind,
                default,
                value,
            },
            float,
        ))
    }

    impl Usb {
        fn reported(&self) -> Vec<(KnownCameraControl, controls::Control, bool)> {
            let mut out: Vec<_> = self
                .camera
                .camera_controls()
                .unwrap_or_default()
                .iter()
                .filter_map(|c| offered(c).map(|(o, float)| (c.control(), o, float)))
                .collect();
            out.sort_by_key(|(_, c, _)| controls::KEYS.iter().position(|k| *k == c.key));
            out
        }
    }

    impl Grab for Usb {
        fn grab(&mut self) -> Result<Picture, String> {
            let frame = self.camera.frame().map_err(|e| e.to_string())?;
            let res = frame.resolution();
            to_picture(
                frame.source_frame_format(),
                res.width(),
                res.height(),
                frame.buffer(),
            )
        }

        fn controls(&mut self) -> Vec<controls::Control> {
            self.reported().into_iter().map(|(_, c, _)| c).collect()
        }

        fn apply(&mut self, settings: &BTreeMap<String, f64>) -> Vec<String> {
            let mut refused = Vec::new();
            for (id, c, float) in self.reported() {
                let Some(wanted) = settings.get(c.key) else {
                    continue;
                };
                let v = c.fit(*wanted);
                let setter = match (c.kind, float) {
                    (Kind::Switch, _) => ControlValueSetter::Boolean(v >= 0.5),
                    (_, true) => ControlValueSetter::Float(v),
                    (_, false) => ControlValueSetter::Integer(v.round() as i64),
                };
                if self.camera.set_camera_control(id, setter).is_err() {
                    refused.push(format!(
                        "{}: the camera refused {}",
                        controls::title(c.key),
                        lasercraft_plan::num(v)
                    ));
                }
            }
            refused
        }
    }

    fn clamp(v: f64) -> u8 {
        v.round().clamp(0.0, 255.0) as u8
    }

    /// BT.601 limited-range YCbCr to RGB.
    fn yuv(y: u8, u: u8, v: u8) -> [u8; 3] {
        let (c, d, e) = (
            f64::from(y) - 16.0,
            f64::from(u) - 128.0,
            f64::from(v) - 128.0,
        );
        [
            clamp(1.164 * c + 1.596 * e),
            clamp(1.164 * c - 0.392 * d - 0.813 * e),
            clamp(1.164 * c + 2.017 * d),
        ]
    }

    /// A camera frame in the formats USB cameras send, as a picture.
    pub(super) fn to_picture(
        format: FrameFormat,
        w: u32,
        h: u32,
        data: &[u8],
    ) -> Result<Picture, String> {
        let n = w as usize * h as usize;
        let short = || format!("a {format} frame of {w} × {h} is too short");
        let rgb = match format {
            FrameFormat::MJPEG => return Picture::decode(data).map_err(|e| e.0),
            FrameFormat::RAWRGB => data.get(..n * 3).ok_or_else(short)?.to_vec(),
            FrameFormat::RAWBGR => data
                .get(..n * 3)
                .ok_or_else(short)?
                .chunks(3)
                .flat_map(|p| [p[2], p[1], p[0]])
                .collect(),
            FrameFormat::GRAY => data
                .get(..n)
                .ok_or_else(short)?
                .iter()
                .flat_map(|g| [*g; 3])
                .collect(),
            FrameFormat::YUYV => data
                .get(..n * 2)
                .ok_or_else(short)?
                .chunks(4)
                .flat_map(|q| {
                    let (a, b) = (yuv(q[0], q[1], q[3]), yuv(q[2], q[1], q[3]));
                    [a[0], a[1], a[2], b[0], b[1], b[2]]
                })
                .collect(),
            FrameFormat::NV12 => {
                let (y, uv) = data.get(..n + n / 2).ok_or_else(short)?.split_at(n);
                let (wu, mut out) = (w as usize, Vec::with_capacity(n * 3));
                for (i, &luma) in y.iter().enumerate() {
                    let (row, col) = (i / wu, i % wu);
                    let k = (row / 2) * wu + (col / 2) * 2;
                    out.extend(yuv(luma, uv[k], uv[k + 1]));
                }
                out
            }
        };
        Picture::new(w, h, rgb).map_err(|e| e.0)
    }

    /// The USB cameras the system lists: index and name.
    pub(crate) fn list() -> Vec<(u32, String)> {
        let Some(backend) = nokhwa::native_api_backend() else {
            return Vec::new();
        };
        nokhwa::query(backend)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|info| match info.index() {
                CameraIndex::Index(i) => Some((*i, info.human_name())),
                CameraIndex::String(_) => None,
            })
            .collect()
    }
}

/// Every camera of the project's device profile, with its worker.
pub struct Cameras {
    workers: BTreeMap<String, Worker>,
    scene: Arc<Mutex<SimScene>>,
    overlay: Option<Arc<Overlay>>,
    overlay_revision: u64,
    taken: Option<Instant>,
    head_moved: bool,
    usb: Vec<(u32, String)>,
    /// Pictures per second while a camera is shown.
    rate: f64,
    repaint: egui::Context,
    /// Pictures kept for the lens correction.
    pub captures: Vec<LensCapture>,
    /// Every model fitted to them.
    pub fits: Vec<Result<Lens, String>>,
    /// Marks found automatically.
    pub marks: Option<Result<[Point; 4], String>>,
}

impl Cameras {
    pub fn new(ctx: &egui::Context) -> Self {
        #[cfg(target_os = "macos")]
        nokhwa::nokhwa_initialize(|_| {});
        Self {
            workers: BTreeMap::new(),
            scene: Arc::new(Mutex::new(SimScene::new(WorkArea::default()))),
            overlay: None,
            overlay_revision: 0,
            taken: None,
            head_moved: false,
            usb: Vec::new(),
            rate: 2.0,
            repaint: ctx.clone(),
            captures: Vec::new(),
            fits: Vec::new(),
            marks: None,
        }
    }

    /// Starts and stops workers so they match `cameras`; `live` is false
    /// when no camera view is shown (the workers then stop).
    pub fn sync(&mut self, cameras: &[Camera], area: &WorkArea, live: bool) {
        let new_bed = self.scene.lock().is_ok_and(|mut scene| {
            let changed = scene.area != *area;
            if changed {
                *scene = SimScene::new(*area);
            }
            changed
        });
        if new_bed {
            self.clear_overlay();
        }
        let wanted: Vec<&Camera> = if live {
            cameras.iter().collect()
        } else {
            Vec::new()
        };
        self.workers.retain(|name, w| {
            let keep = wanted
                .iter()
                .any(|c| &c.name == name && c.source == w.source);
            if !keep {
                let _ = w.control.send(Control::Stop);
            }
            keep
        });
        for c in wanted {
            if let Some(w) = self.workers.get_mut(&c.name) {
                if w.settings != c.controls {
                    w.settings.clone_from(&c.controls);
                    let _ = w.control.send(Control::Settings(c.controls.clone()));
                }
                continue;
            }
            let (control, messages) = spawn(c, self.rate, &self.scene, &self.repaint);
            self.workers.insert(
                c.name.clone(),
                Worker {
                    source: c.source.clone(),
                    control,
                    messages,
                    status: CameraStatus::Connecting,
                    latest: None,
                    revision: 0,
                    settings: c.controls.clone(),
                    offered: Vec::new(),
                },
            );
        }
    }

    /// Collects the workers' pictures and failures; returns notices for
    /// the log.
    pub fn poll(&mut self) -> Vec<String> {
        let mut notices = Vec::new();
        for (name, w) in &mut self.workers {
            while let Ok(m) = w.messages.try_recv() {
                match m {
                    Message::Picture(p) => {
                        w.status = CameraStatus::Connected {
                            width: p.width(),
                            height: p.height(),
                        };
                        w.latest = Some(Arc::new(p));
                        w.revision += 1;
                    }
                    Message::Failed(e) => w.status = CameraStatus::Failed(e),
                    Message::Controls(c) => w.offered = c,
                    Message::Notice(n) => notices.push(format!("camera \"{name}\": {n}")),
                }
            }
        }
        notices
    }

    /// Changes the preview rate of every camera.
    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate;
        for w in self.workers.values() {
            let _ = w.control.send(Control::Rate(rate));
        }
    }

    /// The latest picture of a camera.
    pub fn picture(&self, name: &str) -> Option<Arc<Picture>> {
        self.workers.get(name).and_then(|w| w.latest.clone())
    }

    /// Rebuilds the overlay from every used, aligned camera's latest
    /// picture; with `region`, only that part (bed mm).
    pub fn capture(
        &mut self,
        cameras: &[Camera],
        area: &WorkArea,
        region: Option<Rect>,
    ) -> Result<(), String> {
        let mut overlay = match (&self.overlay, region) {
            (Some(o), Some(_)) => (**o).clone(),
            _ => Overlay::empty(area.width_mm, area.height_mm),
        };
        let mut used = 0;
        for c in cameras.iter().filter(|c| c.used) {
            let Some(alignment) = c.alignment() else {
                continue;
            };
            let Some(picture) = self.picture(&c.name) else {
                continue;
            };
            let lens = c.lens_or_plain(picture.width(), picture.height());
            let mut one = Overlay::empty(area.width_mm, area.height_mm);
            one.paint(&picture, &lens, alignment, &c.fit, region);
            // Opacity per camera, then laid over the cameras before it.
            let alpha = (c.opacity / 100.0 * 255.0).round() as u8;
            for (dst, src) in overlay.rgba.chunks_mut(4).zip(one.rgba.chunks(4)) {
                if src[3] > 0 {
                    dst.copy_from_slice(&[src[0], src[1], src[2], alpha]);
                }
            }
            used += 1;
        }
        if used == 0 {
            return Err(
                "No camera picture to show: a camera needs a picture and a bed alignment."
                    .to_owned(),
            );
        }
        self.overlay = Some(Arc::new(overlay));
        self.overlay_revision += 1;
        self.taken = Some(Instant::now());
        self.head_moved = false;
        Ok(())
    }

    /// Forgets the overlay (another bed).
    fn clear_overlay(&mut self) {
        self.overlay = None;
        self.overlay_revision += 1;
        self.taken = None;
    }

    /// The head moved (a jog, frame or job): the picture may be stale.
    pub fn head_moved(&mut self) {
        self.head_moved = true;
    }

    /// A program streamed to the simulated laser: the simulated camera
    /// sees what it burns.
    pub fn simulated_burn(&mut self, gcode: &str) {
        if let Ok(mut scene) = self.scene.lock() {
            scene.burn_gcode(gcode);
        }
    }

    /// Shows the calibration sheet to the simulated camera at one of the
    /// lens wizard's five places, or hides it.
    pub fn simulated_sheet(&mut self, place: Option<usize>) {
        if let Ok(mut scene) = self.scene.lock() {
            scene.sheet = place.map(|i| sheet_positions(&scene.area)[i % 5]);
        }
    }

    /// Lists USB cameras again.
    pub fn refresh_usb(&mut self) {
        self.usb = usb::list();
    }

    /// What the shell draws.
    pub fn view(&self, cameras: &[Camera]) -> CameraView {
        CameraView {
            cameras: cameras
                .iter()
                .map(|c| {
                    let w = self.workers.get(&c.name);
                    CameraLive {
                        name: c.name.clone(),
                        status: w.map_or(CameraStatus::Off, |w| w.status.clone()),
                        picture: w.and_then(|w| w.latest.clone()),
                        revision: w.map_or(0, |w| w.revision),
                        controls: w.map_or_else(Vec::new, |w| w.offered.clone()),
                    }
                })
                .collect(),
            overlay: self.overlay.clone(),
            overlay_revision: self.overlay_revision,
            picture_age: self.taken.map(|t| t.elapsed().as_secs()),
            stale: self.head_moved,
            usb: self.usb.iter().map(|(i, n)| format!("{i}: {n}")).collect(),
            rate: self.rate,
            captures: self.captures.clone(),
            fits: self.fits.clone(),
            marks: self.marks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usb_frames_become_rgb_pictures() {
        use nokhwa::utils::FrameFormat;
        let gray = usb::to_picture(FrameFormat::GRAY, 2, 1, &[0, 255]).unwrap();
        assert_eq!(gray.get(1, 0), [255, 255, 255]);
        let bgr = usb::to_picture(FrameFormat::RAWBGR, 1, 1, &[1, 2, 3]).unwrap();
        assert_eq!(bgr.get(0, 0), [3, 2, 1]);
        // Y 235 with neutral chroma is white in limited range.
        let yuyv = usb::to_picture(FrameFormat::YUYV, 2, 1, &[235, 128, 16, 128]).unwrap();
        assert_eq!(yuyv.get(0, 0), [255, 255, 255]);
        assert_eq!(yuyv.get(1, 0), [0, 0, 0]);
        let nv12 = usb::to_picture(FrameFormat::NV12, 2, 2, &[235, 235, 16, 16, 128, 128]).unwrap();
        assert_eq!(nv12.get(1, 1), [0, 0, 0]);
        assert!(usb::to_picture(FrameFormat::YUYV, 4, 4, &[0; 3]).is_err());
        let jpeg_or_png = Picture::filled(3, 2, [9, 8, 7]).encode_png();
        assert!(usb::to_picture(FrameFormat::MJPEG, 3, 2, &jpeg_or_png).is_ok());
    }

    #[test]
    fn the_simulated_camera_sees_burned_gcode_and_the_sheet() {
        let area = WorkArea::default();
        let mut scene = SimScene::new(area);
        // A front-left bed 400 deep: machine Y 300 is document y 100.
        scene.burn_gcode(
            "G90 G21\nM4 S0\nG0 X100 Y300\nG1 X200 Y300 S500 F600\nG0 X0 Y0\nM5\nG1 X50 Y50 S0\n",
        );
        assert_eq!(scene.color(Point::new(150.0, 100.0)), Some([45, 32, 20]));
        assert_ne!(scene.color(Point::new(150.0, 110.0)), Some([45, 32, 20]));
        assert_ne!(
            scene.color(Point::new(25.0, 375.0)),
            Some([45, 32, 20]),
            "laser off"
        );
        // Every lens wizard place shows a sheet the dot finder can use.
        let (lens, pose) = simulated_camera(&area);
        let renderer = Renderer::new(&lens, &pose);
        for place in sheet_positions(&area) {
            scene.sheet = Some(place);
            let picture = renderer.render([70, 70, 74], |p| scene.color(p));
            let dots = lasercraft_camera::dots::find_sheet(&picture).unwrap();
            assert!(dots.len() >= 40, "{place:?}: {}", dots.len());
        }
        let mut cut = Vec::new();
        assert!(
            next_jpeg(&mut &b"xx\xFF\xD8abc\xFF\xD9yy"[..])
                .map(|j| cut = j)
                .is_ok()
        );
        assert_eq!(cut, b"\xFF\xD8abc\xFF\xD9");
    }

    #[test]
    fn the_simulated_camera_offers_and_applies_picture_settings() {
        let scene = Arc::new(Mutex::new(SimScene::new(WorkArea::default())));
        let mut camera = Simulated::new(scene);
        let plain = camera.grab().unwrap();
        let keys: Vec<&str> = camera.controls().iter().map(|c| c.key).collect();
        assert_eq!(keys, ["brightness", "contrast"]);
        let settings = BTreeMap::from([("brightness".to_owned(), 250.0)]);
        assert!(camera.apply(&settings).is_empty());
        assert_eq!(camera.controls()[0].value, 100.0, "clamped to the range");
        let bright = camera.grab().unwrap();
        assert!(bright.get(5, 5)[0] > plain.get(5, 5)[0]);
        camera.apply(&BTreeMap::new());
        assert_eq!(camera.grab().unwrap(), plain, "back to the defaults");
    }
}
