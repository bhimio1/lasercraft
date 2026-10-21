//! What to tell the user when the OS refuses to open a serial port
//! (`docs/linux-serial.md`).

/// Linux distribution families whose serial device group differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxFamily {
    /// Arch and derivatives (CachyOS, Manjaro, EndeavourOS): group `uucp`.
    Arch,
    /// Debian and derivatives (Ubuntu, Mint, Pop!_OS): group `dialout`.
    Debian,
    /// Fedora and RHEL-like: group `dialout`.
    Fedora,
    /// Anything else.
    Other,
}

impl LinuxFamily {
    /// Guesses the family from the `ID` and `ID_LIKE` fields of
    /// `/etc/os-release` text.
    #[must_use]
    pub fn from_os_release(text: &str) -> Self {
        let ids: Vec<String> = text
            .lines()
            .filter_map(|line| {
                let (key, value) = line.split_once('=')?;
                matches!(key.trim(), "ID" | "ID_LIKE")
                    .then(|| value.trim().trim_matches('"').to_ascii_lowercase())
            })
            .flat_map(|v| v.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
            .collect();
        let has = |name: &str| ids.iter().any(|id| id == name);
        if has("arch") {
            Self::Arch
        } else if has("debian") || has("ubuntu") {
            Self::Debian
        } else if has("fedora") || has("rhel") {
            Self::Fedora
        } else {
            Self::Other
        }
    }

    /// The group that owns serial devices.
    #[must_use]
    pub const fn serial_group(self) -> &'static str {
        match self {
            Self::Arch => "uucp",
            Self::Debian | Self::Fedora | Self::Other => "dialout",
        }
    }
}

/// Instructions for fixing "permission denied" on `port` under Linux.
#[must_use]
pub fn linux_help(port: &str, family: LinuxFamily, in_flatpak: bool) -> String {
    let group = family.serial_group();
    let mut help = format!(
        "LaserCraft is not allowed to open {port}. On Linux, serial ports belong to the \
         `{group}` group. Add yourself to it, then log out and back in:\n\n    \
         sudo usermod -aG {group} $USER\n\nCheck the port's group with `ls -l {port}`."
    );
    if family == LinuxFamily::Other {
        help.push_str(" If the group shown there is not `dialout`, use that group instead.");
    }
    if in_flatpak {
        help.push_str(
            "\n\nThis is the Flatpak build. It needs device access, which it requests by \
             default; if you removed it, restore it with:\n\n    \
             flatpak override --user --device=all io.github.bhimio1.LaserCraft",
        );
    }
    help
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_arch_derivatives() {
        let cachy = "NAME=\"CachyOS Linux\"\nID=cachyos\nID_LIKE=arch\n";
        assert_eq!(LinuxFamily::from_os_release(cachy), LinuxFamily::Arch);
        assert_eq!(LinuxFamily::from_os_release("ID=arch\n"), LinuxFamily::Arch);
    }

    #[test]
    fn detects_debian_family_and_fedora() {
        let mint = "ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
        assert_eq!(LinuxFamily::from_os_release(mint), LinuxFamily::Debian);
        assert_eq!(
            LinuxFamily::from_os_release("ID=fedora\n"),
            LinuxFamily::Fedora
        );
        assert_eq!(
            LinuxFamily::from_os_release("ID=nixos\n"),
            LinuxFamily::Other
        );
    }

    #[test]
    fn help_names_the_right_group() {
        let arch = linux_help("/dev/ttyUSB0", LinuxFamily::Arch, false);
        assert!(arch.contains("usermod -aG uucp"));
        assert!(arch.contains("/dev/ttyUSB0"));
        assert!(!arch.contains("flatpak"));
        let deb = linux_help("/dev/ttyACM0", LinuxFamily::Debian, true);
        assert!(deb.contains("usermod -aG dialout"));
        assert!(deb.contains("--device=all"));
    }
}
