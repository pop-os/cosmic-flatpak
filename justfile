repo:
    #!/usr/bin/env bash
    set -ex
    ls -1 app | while read ref
    do
        flatpak-builder \
            --ccache \
            --force-clean \
            --gpg-sign="${DEBEMAIL}" \
            --repo=repo \
            --sandbox \
            --user \
            --verbose \
            "build/app/${ref}" \
            "app/${ref}/${ref}.json"
    done
    touch repo