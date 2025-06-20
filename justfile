repo:
    #!/usr/bin/env bash

    set -ex

    function fb {
        flatpak-builder \
            --ccache \
            --force-clean \
            --gpg-sign="${DEBEMAIL}" \
            --repo=repo \
            --sandbox \
            --user \
            --verbose \
            "$@"
    }

    # Initialize repo
    if [ ! -e repo ]
    then
        ostree --repo=repo init --mode=archive
    fi

    # Build freedesktop sdk
    for id_branch in \
        org.freedesktop.Sdk/24.08
    do
        ln -srf repo "dep/${id_branch}/repo"
        make -C "dep/${id_branch}" export
    done

    # Build other dependencies
    for id_branch in \
        org.freedesktop.Sdk.Extension.rust-stable/24.08 \
        com.system76.Cosmic.BaseApp/stable
    do
        id="$(dirname "${id_branch}")"
        fb "build/dep/${id_branch}" "dep/${id_branch}/${id}.json"
    done

    # Build all apps
    ls -1 app | while read id
    do
        fb "build/app/${id}" "app/${id}/${id}.json"
    done

    # Generate update information and appstream data
    flatpak \
        build-update-repo \
        --gpg-sign="${DEBEMAIL}" \
        --generate-static-deltas \
        --prune \
        repo

clean:
    rm -rf .flatpak-builder build repo

ubuntu-deps:
    sudo apt-get install --yes \
        flatpak \
        flatpak-builder \
        pipx
    pipx install 'BuildStream == 2.*'
    pipx inject buildstream \
        dulwich \
        requests \
        tomli \
        tomlkit
