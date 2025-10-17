# COSMIC Flatpak Repository

This repository hosts applets and other flatpaks for COSMIC that are not suitable for upload
to Flathub. For COSMIC apps, please first try to submit to Flathub by following these instructions: https://docs.flathub.org/docs/for-app-authors/submission

To install this repository, run the following commands:
```
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
```

To test building an app, run the following commands:
```
# Install dependencies (only run once):
sudo apt-get install flatpak flatpak-builder just
# Build app with ID com.example.app (replace ID with yours)
just build com.example.app
```
