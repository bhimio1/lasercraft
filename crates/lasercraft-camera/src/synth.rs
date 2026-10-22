//! A synthetic camera (own design): renders a flat scene through a known
//! lens and pose. Tests use it to check calibration and alignment against
//! the truth; the app's simulated camera uses it too.

use lasercraft_geom::Point;

use crate::lens::Lens;
use crate::picture::Picture;

/// A rotation and a camera center: the camera looks along its +z axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// World-to-camera rotation, row-major.
    pub r: [f64; 9],
    /// Camera center in world mm (the scene plane is z = 0).
    pub center: [f64; 3],
}

/// The rotation for a Rodrigues vector (axis × angle).
#[must_use]
pub fn rodrigues(w: [f64; 3]) -> [f64; 9] {
    let theta = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
    if theta < 1e-15 {
        return [1.0, -w[2], w[1], w[2], 1.0, -w[0], -w[1], w[0], 1.0];
    }
    let (kx, ky, kz) = (w[0] / theta, w[1] / theta, w[2] / theta);
    let (s, c) = theta.sin_cos();
    let v = 1.0 - c;
    [
        c + kx * kx * v,
        kx * ky * v - kz * s,
        kx * kz * v + ky * s,
        ky * kx * v + kz * s,
        c + ky * ky * v,
        ky * kz * v - kx * s,
        kz * kx * v - ky * s,
        kz * ky * v + kx * s,
        c + kz * kz * v,
    ]
}

impl Pose {
    /// A camera `height` mm above `(x, y)` looking straight down, picture
    /// up towards world −y, tilted by the small angles `tilt` (radians,
    /// Rodrigues vector).
    #[must_use]
    pub fn overhead(x: f64, y: f64, height: f64, tilt: [f64; 3]) -> Self {
        Self {
            r: rodrigues(tilt),
            center: [x, y, -height],
        }
    }

    /// A world point in camera coordinates.
    #[must_use]
    pub fn to_camera(&self, p: [f64; 3]) -> [f64; 3] {
        let d = [
            p[0] - self.center[0],
            p[1] - self.center[1],
            p[2] - self.center[2],
        ];
        let r = &self.r;
        [
            r[0] * d[0] + r[1] * d[1] + r[2] * d[2],
            r[3] * d[0] + r[4] * d[1] + r[5] * d[2],
            r[6] * d[0] + r[7] * d[1] + r[8] * d[2],
        ]
    }

    /// The ideal ray of a point on the plane, `None` behind the camera.
    #[must_use]
    pub fn ray(&self, p: Point) -> Option<Point> {
        let c = self.to_camera([p.x, p.y, 0.0]);
        (c[2] > 1e-9).then(|| Point::new(c[0] / c[2], c[1] / c[2]))
    }

    /// Where an ideal ray meets the plane, `None` when it does not.
    #[must_use]
    pub fn hit(&self, ray: Point) -> Option<Point> {
        let r = &self.r;
        // Camera-to-world direction: Rᵀ · (x, y, 1).
        let d = [
            r[0] * ray.x + r[3] * ray.y + r[6],
            r[1] * ray.x + r[4] * ray.y + r[7],
            r[2] * ray.x + r[5] * ray.y + r[8],
        ];
        if d[2] <= 1e-12 {
            return None;
        }
        let s = -self.center[2] / d[2];
        Some(Point::new(
            self.center[0] + s * d[0],
            self.center[1] + s * d[1],
        ))
    }
}

/// A camera that keeps, for each of 2 × 2 samples per pixel, the plane
/// point it sees: rendering a scene is then cheap.
#[derive(Debug, Clone)]
pub struct Renderer {
    width: u32,
    height: u32,
    hits: Vec<Option<Point>>,
}

impl Renderer {
    /// The plane points `lens` sees from `pose`.
    #[must_use]
    pub fn new(lens: &Lens, pose: &Pose) -> Self {
        let mut hits = Vec::with_capacity(lens.width as usize * lens.height as usize * 4);
        for y in 0..lens.height {
            for x in 0..lens.width {
                for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    let pixel = Point::new(f64::from(x) + dx, f64::from(y) + dy);
                    hits.push(pose.hit(lens.unproject(pixel)));
                }
            }
        }
        Self {
            width: lens.width,
            height: lens.height,
            hits,
        }
    }

    /// Renders `scene` (a color per plane point, `None` for background).
    pub fn render(&self, background: [u8; 3], scene: impl Fn(Point) -> Option<[u8; 3]>) -> Picture {
        let mut out = Picture::filled(self.width, self.height, background);
        for (i, samples) in self.hits.as_chunks::<4>().0.iter().enumerate() {
            let mut sum = [0u32; 3];
            for hit in samples {
                let c = hit.and_then(&scene).unwrap_or(background);
                for k in 0..3 {
                    sum[k] += u32::from(c[k]);
                }
            }
            let (x, y) = (i as u32 % self.width, i as u32 / self.width);
            out.set(x, y, sum.map(|s| ((s + 2) / 4) as u8));
        }
        out
    }
}

/// Renders `scene` (a color per plane point, `None` for background) as
/// the camera sees it, with 2 × 2 samples per pixel.
pub fn render(
    lens: &Lens,
    pose: &Pose,
    background: [u8; 3],
    scene: impl Fn(Point) -> Option<[u8; 3]>,
) -> Picture {
    Renderer::new(lens, pose).render(background, scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::Model;

    #[test]
    fn rays_and_hits_are_inverse() {
        let pose = Pose::overhead(150.0, 100.0, 400.0, [0.05, -0.03, 0.2]);
        for p in [
            Point::new(150.0, 100.0),
            Point::new(10.0, 190.0),
            Point::new(290.0, 5.0),
        ] {
            let back = pose.hit(pose.ray(p).unwrap()).unwrap();
            assert!(back.distance(p) < 1e-9, "{p:?} {back:?}");
        }
        // Straight down: the bed center is the picture center.
        let lens = Lens::ideal(Model::Standard, 64, 48, 60.0);
        let flat = Pose::overhead(150.0, 100.0, 400.0, [0.0; 3]);
        let c = lens.project(flat.ray(Point::new(150.0, 100.0)).unwrap());
        assert!((c.x - 32.0).abs() < 1e-9 && (c.y - 24.0).abs() < 1e-9);
        let pic = render(&lens, &flat, [255, 255, 255], |p| {
            (p.x < 150.0).then_some([0, 0, 0])
        });
        assert_eq!(pic.get(10, 10), [0, 0, 0]);
        assert_eq!(pic.get(50, 10), [255, 255, 255]);
    }
}
