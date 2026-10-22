//! The right dock (`docs/spec/style/layout.md`, docking stage a): an upper
//! group with Layers, Jog and Console tabs and a lower group with Machine
//! and Materials tabs, each with its tabs along its bottom; which panels
//! are shown and which tab of each group is in front. UI state only, never
//! saved in the project.

use crate::theme::{Tokens, size, weight};

/// A dockable panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPanel {
    /// The layer list and editor.
    Layers,
    /// Head position, jog pad and user origin.
    Jog,
    /// Messages from GRBL.
    Console,
    /// Connection and job control.
    Machine,
    /// Material libraries (M8).
    Materials,
}

impl DockPanel {
    /// Every panel, in Window → Panels order (alphabetical).
    pub const ALL: [Self; 5] = [
        Self::Console,
        Self::Jog,
        Self::Layers,
        Self::Machine,
        Self::Materials,
    ];
    /// The upper group's tabs, left to right.
    pub const UPPER: [Self; 3] = [Self::Layers, Self::Jog, Self::Console];
    /// The lower group's tabs, left to right.
    pub const LOWER: [Self; 2] = [Self::Machine, Self::Materials];

    /// The panel's name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Layers => "Layers",
            Self::Jog => "Jog",
            Self::Console => "Console",
            Self::Machine => "Machine",
            Self::Materials => "Materials",
        }
    }

    /// What the panel is for (Window → Panels tooltips).
    #[must_use]
    pub fn about(self) -> &'static str {
        match self {
            Self::Layers => "Layer list and settings",
            Self::Jog => "Move the laser head by hand and store origins",
            Self::Console => "Messages from GRBL and a line for raw commands",
            Self::Machine => "Laser connection and job control",
            Self::Materials => "Material libraries: stored layer settings",
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    fn upper(self) -> bool {
        Self::UPPER.contains(&self)
    }
}

/// Which panels are shown, which tab of each group is in front, and the
/// other show/hide state of the main window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dock {
    shown: [bool; 5],
    front: [DockPanel; 2],
    /// The command line's log is open.
    pub log_open: bool,
    /// The palette strip is shown.
    pub palette: bool,
    /// The drawing tools are shown.
    pub tools: bool,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            shown: [true; 5],
            front: [DockPanel::Layers, DockPanel::Machine],
            log_open: false,
            palette: true,
            tools: true,
        }
    }
}

impl Dock {
    /// Whether `panel` is shown.
    #[must_use]
    pub fn is_shown(&self, panel: DockPanel) -> bool {
        self.shown[panel.index()]
    }

    fn group(panel: DockPanel) -> usize {
        usize::from(!panel.upper())
    }

    /// Shows `panel` and brings it to the front of its group.
    pub fn show(&mut self, panel: DockPanel) {
        self.shown[panel.index()] = true;
        self.front[Self::group(panel)] = panel;
    }

    /// Hides `panel`; the next shown tab of its group comes to the front.
    pub fn hide(&mut self, panel: DockPanel) {
        self.shown[panel.index()] = false;
        let g = Self::group(panel);
        if self.front[g] == panel
            && let Some(next) = self.tabs(g == 0).first()
        {
            self.front[g] = *next;
        }
    }

    /// The shown tabs of the upper (or lower) group.
    #[must_use]
    pub fn tabs(&self, upper: bool) -> Vec<DockPanel> {
        let all: &[DockPanel] = if upper {
            &DockPanel::UPPER
        } else {
            &DockPanel::LOWER
        };
        all.iter().copied().filter(|p| self.is_shown(*p)).collect()
    }

    /// The front tab of the upper (or lower) group, if any tab is shown.
    #[must_use]
    pub fn front_of(&self, upper: bool) -> Option<DockPanel> {
        let tabs = self.tabs(upper);
        let front = self.front[usize::from(!upper)];
        if tabs.contains(&front) {
            Some(front)
        } else {
            tabs.first().copied()
        }
    }

    /// Whether any dock panel is shown.
    #[must_use]
    pub fn any(&self) -> bool {
        self.shown.iter().any(|s| *s)
    }

    /// Back to the default arrangement.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// A group's title strip: the front panel's name and a close cross.
/// Returns whether the cross was clicked.
pub fn title_strip(ui: &mut egui::Ui, name: &str) -> bool {
    let t = Tokens::of(ui.ctx());
    let mut close = false;
    egui::Frame::new()
        .fill(t.panel_title)
        .inner_margin(egui::Margin::symmetric(size::GAP as i8, 1))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cross = egui::Button::new(egui::RichText::new("×").color(t.text_muted))
                        .frame(false);
                    close = ui
                        .add(cross)
                        .on_hover_text("Hide this panel (Window → Panels brings it back)")
                        .clicked();
                });
            });
        });
    close
}

/// The tab strip along the bottom of a group. Returns a clicked tab.
pub fn tab_strip(ui: &mut egui::Ui, tabs: &[DockPanel], front: DockPanel) -> Option<DockPanel> {
    let t = Tokens::of(ui.ctx());
    let mut picked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for tab in tabs {
            let is_front = *tab == front;
            let (fill, color) = if is_front {
                (t.panel, t.text)
            } else {
                (t.window, t.text_muted)
            };
            let button = egui::Button::new(egui::RichText::new(tab.name()).color(color))
                .fill(fill)
                .stroke(egui::Stroke::new(weight::HAIRLINE, t.border))
                .corner_radius(0.0)
                .min_size(egui::vec2(0.0, size::ROW));
            let response = ui.add(button).on_hover_text(tab.about());
            if is_front {
                let r = response.rect;
                ui.painter().line_segment(
                    [
                        r.left_bottom() + egui::vec2(4.0, -1.0),
                        r.right_bottom() + egui::vec2(-4.0, -1.0),
                    ],
                    egui::Stroke::new(weight::NORMAL, t.accent),
                );
            }
            if response.clicked() {
                picked = Some(*tab);
            }
        }
    });
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiding_the_front_tab_brings_the_next_forward() {
        let mut dock = Dock::default();
        assert_eq!(dock.front_of(true), Some(DockPanel::Layers));
        dock.show(DockPanel::Console);
        assert_eq!(dock.front_of(true), Some(DockPanel::Console));
        dock.hide(DockPanel::Console);
        assert_eq!(dock.front_of(true), Some(DockPanel::Layers));
        for p in DockPanel::UPPER {
            dock.hide(p);
        }
        assert_eq!(dock.front_of(true), None);
        assert!(dock.any(), "Machine is still shown");
        assert_eq!(dock.front_of(false), Some(DockPanel::Machine));
        dock.show(DockPanel::Materials);
        assert_eq!(dock.front_of(false), Some(DockPanel::Materials));
        dock.hide(DockPanel::Materials);
        assert_eq!(dock.front_of(false), Some(DockPanel::Machine));
        dock.show(DockPanel::Materials);
        dock.show(DockPanel::Machine);
        dock.hide(DockPanel::Machine);
        assert_eq!(dock.front_of(false), Some(DockPanel::Materials));
        dock.hide(DockPanel::Materials);
        assert!(!dock.any());
        dock.reset();
        assert_eq!(dock, Dock::default());
    }
}
