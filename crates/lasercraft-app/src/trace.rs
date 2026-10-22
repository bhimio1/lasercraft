//! Answers the Trace to vectors window (`docs/spec/trace.md`, "In the
//! app"): the bitmap of the source, its trace, and the picture to show.

use std::sync::Arc;

use lasercraft_camera::Picture;
use lasercraft_camera::overlay::Overlay;
use lasercraft_commands::trace;
use lasercraft_project::Project;
use lasercraft_project::raster::{Adjust, Gray};
use lasercraft_ui::{TraceInfo, TraceRequest, TraceSource};

/// Largest side of the picture the window shows.
const SHOWN: u32 = 1600;

/// The overlay as a bitmap: what the cameras see, white where none does.
pub fn overlay_gray(overlay: &Overlay) -> Option<Gray> {
    let px = overlay
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            if p[3] == 0 {
                255
            } else {
                let y = 0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2]);
                y.round().clamp(0.0, 255.0) as u8
            }
        })
        .collect();
    Gray::new(overlay.width, overlay.height, px).ok()
}

/// The bitmap a request traces, with its adjustments.
fn source(
    request: &TraceRequest,
    project: &Project,
    overlay: Option<&Overlay>,
) -> Result<(Gray, Adjust), String> {
    match request.source {
        TraceSource::Image(id) => project
            .shape(id)
            .and_then(|s| s.geometry().image())
            .map(|i| ((*i.bitmap).clone(), i.adjust))
            .ok_or_else(|| "The image is gone.".to_owned()),
        TraceSource::Camera => overlay
            .and_then(overlay_gray)
            .map(|g| (g, Adjust::default()))
            .ok_or_else(|| "Capture a camera picture first.".to_owned()),
    }
}

/// The picture the window shows: the adjusted bitmap, shrunk to fit.
fn shown(gray: &Gray, adjust: &Adjust) -> Option<Picture> {
    let k = gray.width().max(gray.height()).div_ceil(SHOWN).max(1);
    let small = gray.shrink(k);
    let table = adjust.table();
    let rgb = small
        .pixels()
        .iter()
        .flat_map(|v| [table[usize::from(*v)]; 3])
        .collect();
    Picture::new(small.width(), small.height(), rgb).ok()
}

/// Traces what the window asks for.
pub fn answer(request: &TraceRequest, project: &Project, overlay: Option<&Overlay>) -> TraceInfo {
    let mut info = TraceInfo {
        revision: request.revision,
        ..TraceInfo::default()
    };
    let (gray, adjust) = match source(request, project, overlay) {
        Ok(s) => s,
        Err(e) => {
            info.error = Some(e);
            return info;
        }
    };
    info.size = (gray.width(), gray.height());
    info.picture = shown(&gray, &adjust).map(Arc::new);
    match trace(&gray, &adjust, &request.settings) {
        Ok(traced) => {
            info.counts = (traced.paths.len(), traced.nodes);
            for path in &traced.paths {
                for (points, _) in path.flatten(0.5) {
                    info.loops.push(points);
                }
                info.nodes.extend(path.subpaths.iter().flat_map(|s| {
                    std::iter::once(s.start).chain(s.segments.iter().map(|seg| seg.end()))
                }));
            }
        }
        Err(e) => info.error = Some(e),
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasercraft_commands::TraceSettings;

    #[test]
    fn the_camera_overlay_traces_with_white_where_no_camera_sees() {
        let mut overlay = Overlay::empty(20.0, 10.0);
        let (w, h) = (overlay.width, overlay.height);
        for y in 5..15 {
            for x in 10..30 {
                let i = ((y * w + x) * 4) as usize;
                overlay.rgba[i..i + 4].copy_from_slice(&[10, 10, 10, 255]);
            }
        }
        assert_eq!(overlay_gray(&overlay).unwrap().get(0, 0), 255);
        let request = TraceRequest {
            source: TraceSource::Camera,
            settings: TraceSettings::default(),
            revision: 3,
        };
        let info = answer(&request, &Project::default(), Some(&overlay));
        assert_eq!((info.revision, info.size), (3, (w, h)));
        assert_eq!(info.counts.0, 1);
        assert!(info.error.is_none());
        let none = answer(&request, &Project::default(), None);
        assert!(none.error.is_some());
    }
}
