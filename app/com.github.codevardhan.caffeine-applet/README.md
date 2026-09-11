# Caffeine Applet for [COSMIC DE](https://system76.com/cosmic/)

A third-party applet for the [COSMIC desktop](https://system76.com/cosmic/) that prevents your system from going idle. Not official COSMIC software and not endorsed by System76. Click the coffee icon to toggle an inhibit lock that blocks idle and sleep, keeping your screen on and your machine awake.

Uses the logind D-Bus `Inhibit` interface directly — no child processes, no PID files, crash-safe by design. Works on any system running systemd-logind or elogind.

## Features

- **One-click toggle**: Click the panel icon to open a menu and pick how long to inhibit — 15 minutes, 30 minutes, 1 hour, or indefinitely.
- **Auto-expiry**: Timed sessions release the inhibit lock on their own when time's up — no need to remember to turn it off.
- **Crash-safe**: Uses a file descriptor–based inhibit lock. If the applet crashes, the OS automatically releases the lock.
- **Minimal**: No background processes, no polling, near-zero resource usage.
- **Distro-agnostic**: Works with systemd-logind and elogind.

## Installation

### Flatpak (recommended)

Available via System76's official COSMIC Flatpak repository (not Flathub — this
applet doesn't have a standalone window/use outside a panel, which is outside
Flathub's scope).

```sh
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install --user cosmic com.github.codevardhan.caffeine-applet
```

If you have COSMIC Store installed, it should also be visible there under the
**COSMIC Applets** category, provided you have *both* the `flathub` and
`cosmic` remotes added at **user** scope. System-scope remotes currently
don't surface applets correctly in COSMIC Store
([pop-os/cosmic-store#477](https://github.com/pop-os/cosmic-store/issues/477)),
so if it isn't showing up, add both as `--user` remotes and it should appear.

### From source

A [justfile](./justfile) is included for the [casey/just](https://github.com/casey/just) command runner.

```sh
git clone https://github.com/codevardhan/caffeine-applet.git
cd caffeine-applet
just build-release
sudo just install
```

Then add the applet to a panel in **Settings → Desktop → Panel → Configure panel applets**.

### Uninstall

```sh
# Flatpak
flatpak uninstall --user com.github.codevardhan.caffeine-applet

# Native
sudo just uninstall
# or manually:
sudo rm /usr/bin/caffeine-applet
sudo rm /usr/share/applications/com.github.codevardhan.caffeine-applet.desktop
sudo rm /usr/share/icons/hicolor/scalable/apps/com.github.codevardhan.caffeine-applet*.svg
sudo rm /usr/share/metainfo/com.github.codevardhan.caffeine-applet.metainfo.xml
```

Log out and back in for COSMIC settings to update.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally and build with vendored sources:

```sh
just vendor
just build-vendored
just rootdir=debian/caffeine-applet prefix=/usr install
```

## Translators

[Fluent](https://projectfluent.org/) is used for localization. Translation files are in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en), rename `en` to the desired [ISO 639-1 language code](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes), and provide translations for each message identifier.

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## License

[MPL-2.0](./LICENSE)