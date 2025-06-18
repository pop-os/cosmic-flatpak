repo:
    #!/usr/bin/env bash
    set -ex
    ls -1 app | while read ref
    do
        flatpak-builder \
            --ccache \
            --force-clean \
            --gpg-sign="${DEBEMAIL}" \
            --install-deps-from=flathub \
            --repo=repo \
            --sandbox \
            --user \
            --verbose \
            "build/app/${ref}" \
            "app/${ref}/${ref}.json"
    done
    flatpak \
        build-update-repo \
        --gpg-sign="${DEBEMAIL}" \
        --generate-static-deltas \
        --prune \
        repo
    touch repo
