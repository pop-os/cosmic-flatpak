repo:
    #!/usr/bin/env bash

    set -ex

    # Clear log directory
    rm -rf log
    mkdir -p log/app

    # Build all apps
    ls -1 app | while read id
    do
        flatpak-builder \
            --ccache \
            --force-clean \
            --gpg-sign="${DEBEMAIL}" \
            --install-deps-from=flathub \
            --repo=repo \
            --require-changes \
            --sandbox \
            --user \
            --verbose \
            "build/app/${id}" \
            "app/${id}/${id}.json" \
            | tee "log/app/${id}.txt"
    done

    # Generate update information and appstream data
    flatpak \
        build-update-repo \
        --gpg-sign="${DEBEMAIL}" \
        --generate-static-deltas \
        --prune \
        repo

clean:
    rm -rf .flatpak-builder build log repo

ostree-info:
    #!/usr/bin/env bash
    set -e
    ostree --repo=repo refs | while read ref
    do
        ostree --repo=repo log "${ref}"
    done

ubuntu-deps:
    sudo apt-get install --yes \
        flatpak \
        flatpak-builder