//! The machine and export settings a job uses: the project's device
//! profile's, or the defaults (`docs/spec/devices.md`, "The project's
//! device").

use lasercraft_gcode::{Air, Options};
use lasercraft_plan::Machine;
use lasercraft_project::Project;
use lasercraft_workshop::Workshop;

/// The machine and export options for `project`: from its device profile
/// when it names one that exists, otherwise the defaults. The message says
/// why a named profile was not used.
#[must_use]
pub fn job_settings(project: &Project, workshop: &Workshop) -> (Machine, Options, Option<String>) {
    let mut machine = Machine::default();
    let mut options = Options::default();
    let Some(name) = project.device() else {
        return (machine, options, None);
    };
    let Some(profile) = workshop.profile(name) else {
        return (
            machine,
            options,
            Some(format!(
                "Warning: device profile \"{name}\" is not on this computer; default limits are used."
            )),
        );
    };
    machine.max_speed = profile.max_speed;
    machine.acceleration = profile.acceleration;
    machine.max_power = profile.max_power;
    options.s_max = profile.s_max;
    options.air = profile.air.parse().unwrap_or(Air::M8);
    (machine, options, None)
}

#[cfg(test)]
mod tests {
    use lasercraft_workshop::DeviceProfile;

    use super::*;

    #[test]
    fn the_projects_profile_sets_the_limits() {
        let mut ws = Workshop::in_memory();
        let mut profile = DeviceProfile::new("Diode").unwrap();
        profile.set("max_power", "60").unwrap();
        profile.set("s_max", "255").unwrap();
        profile.set("air", "m7").unwrap();
        ws.add_profile(profile).unwrap();
        let mut p = Project::default();
        let (m, o, w) = job_settings(&p, &ws);
        assert_eq!((m.max_power, o.s_max, w), (100.0, 1000.0, None));
        p.set_device(Some("diode")).unwrap();
        let (m, o, w) = job_settings(&p, &ws);
        assert_eq!(
            (m.max_power, o.s_max, o.air, w),
            (60.0, 255.0, Air::M7, None)
        );
        p.set_device(Some("Gone")).unwrap();
        assert!(
            job_settings(&p, &ws)
                .2
                .unwrap()
                .contains("not on this computer")
        );
    }
}
