# Serial ports on Linux

GRBL controllers appear as USB serial devices, usually `/dev/ttyUSB0` (USB-UART
bridges such as CH340 or CP210x) or `/dev/ttyACM0` (USB CDC-ACM boards). A
normal user may open them only as a member of the group that owns them.

| Distribution | Group | Command |
| --- | --- | --- |
| Arch, CachyOS, Manjaro, EndeavourOS | `uucp` | `sudo usermod -aG uucp $USER` |
| Debian, Ubuntu, Mint, Pop!_OS | `dialout` | `sudo usermod -aG dialout $USER` |
| Fedora | `dialout` | `sudo usermod -aG dialout $USER` |

Log out and back in (or reboot) afterwards. Check with `ls -l /dev/ttyUSB0`:
the group column is the one you need, which matters on other distributions.

LaserCraft recognizes "permission denied" when opening a port and shows this
fix in the app, naming the group for your distribution
(`lasercraft_device::permissions`).

## Flatpak

The Flatpak build needs access to host devices to see serial ports. Its
manifest (added with the release builds in M10) requests `--device=all`, as
Flatpak has no narrower permission for serial ports. If it was removed:

```sh
flatpak override --user --device=all io.github.bhimio1.LaserCraft
```

Group membership still applies inside the sandbox.

## Other notes

- ModemManager can probe new serial devices and confuse a controller for a few
  seconds after plugging in. If connecting fails right after plugging in, wait
  and retry; or disable ModemManager if you do not use a mobile modem.
- Some boards reset when the port is opened (the DTR line). GRBL then prints
  its welcome banner, which LaserCraft waits for (M6).
