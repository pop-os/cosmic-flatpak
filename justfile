repo:
    #!/usr/bin/env bash

    set -ex

    # Build all apps
    arch="$(flatpak --default-arch)"
    ls -1 app | while read id
    do
        mkdir -p "log/app/${id}"
        flatpak-builder \
            --arch="${arch}" \
            --ccache \
            --force-clean \
            --gpg-sign="${DEBEMAIL}" \
            --install-deps-from=flathub \
            --repo=repo \
            --require-changes \
            --sandbox \
            --user \
            --verbose \
            "build/app/${id}/${arch}" \
            "app/${id}/${id}.json" \
            2>&1 | tee "log/app/${id}/${arch}.txt"
    done

    # Generate update information and appstream data
    flatpak \
        build-update-repo \
        --gpg-sign="${DEBEMAIL}" \
        --generate-static-deltas \
        --prune \
        repo

clean:
    rm -rf build log

distclean: clean
    rm -rf .flatpak-builder repo

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
