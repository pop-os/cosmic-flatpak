# Build repo
repo:
    #!/usr/bin/env bash
    set -e

    # Build all apps
    ls -1 app | while read id
    do
        just build ${id}
    done

    cat end-of-life.txt | grep -v '^#' | grep -v '^[[:space:]]*$' | while read eol
    do
        id="$(echo "${eol}" | cut -d '=' -f 1)"
        rebase="$(echo "${eol}" | cut -d '=' -f 2)"
        reason="$(echo "${eol}" | cut -d '=' -f 3-)"
        just eol "${id}" "${rebase}" "${reason}"
    done

    gpg_args=()
    if [ -n "${DEBEMAIL:-}" ]
    then
        gpg_args+=(--gpg-sign="${DEBEMAIL}")
    fi

    # Generate update information and appstream data
    set -x
    flatpak \
        build-update-repo \
        "${gpg_args[@]}" \
        --generate-static-deltas \
        --prune \
        repo

# Build app with specified ID
build id:
    #!/usr/bin/env bash
    set -e
    arch="$(flatpak --default-arch)"
    gpg_args=()
    if [ -n "${DEBEMAIL:-}" ]
    then
        gpg_args+=(--gpg-sign="${DEBEMAIL}")
    fi

    set -x
    mkdir -p "log/app/{{id}}"
    flatpak-builder \
        --arch="${arch}" \
        --ccache \
        --delete-build-dirs \
        --force-clean \
        "${gpg_args[@]}" \
        --install-deps-from=flathub \
        --repo=repo \
        --require-changes \
        --sandbox \
        --user \
        --verbose \
        "build/app/{{id}}/${arch}" \
        "app/{{id}}/{{id}}.json" \
        2>&1 | tee "log/app/{{id}}/${arch}.txt"

# Build manifests changed from origin/master
build-changed:
    #!/usr/bin/env bash
    set -e
    git fetch origin master
    git diff --name-only origin/master...HEAD | grep '^app/' | cut -d / -f2 | sort | uniq | while read id
    do
        # A PR that withdraws an app leaves its ID in the diff with no manifest behind it.
        if [ -f "app/${id}/${id}.json" ]
        then
            just build ${id}
        else
            echo "${id} manifest removed, nothing to build"
        fi
    done

# EOL app with specified id, optional rebase id and optional reason
eol id rebase="" reason="":
    #!/usr/bin/env bash
    set -e
    id="{{id}}"
    rebase="{{rebase}}"
    reason="{{reason}}"
    arch="$(flatpak --default-arch)"
    gpg_args=()
    if [ -n "${DEBEMAIL:-}" ]
    then
        gpg_args+=(--gpg-sign="${DEBEMAIL}")
    fi

    ref="app/${id}/${arch}/master"
    if ! ostree --repo=repo show "${ref}"
    then
        echo "${id} not found, nothing to end of life"
        exit 0
    fi

    eol_args=()
    if [ -n "${rebase}" ]
    then
        # Renamed: mark end of life and redirect existing installs to the new ID.
        current_rebase="$(ostree --repo=repo show --print-metadata-key=ostree.endoflife-rebase "${ref}" || true)"
        if [ "${current_rebase}" == "'app/${rebase}/${arch}/master'" ]
        then
            echo "${id} already rebased to ${rebase}"
            exit 0
        fi
        eol_args+=(--end-of-life="${reason:-Application has been renamed to ${rebase}}")
        eol_args+=(--end-of-life-rebase="${id}=${rebase}")
    else
        # Withdrawn: mark end of life with no replacement to rebase onto. Deleting the
        # manifest from app/ only stops rebuilds; the ref stays in the repo and the app
        # stays installable, so it has to be marked here as well.
        current_eol="$(ostree --repo=repo show --print-metadata-key=ostree.endoflife "${ref}" || true)"
        if [ -n "${current_eol}" ]
        then
            echo "${id} already marked end of life"
            exit 0
        fi
        eol_args+=(--end-of-life="${reason:-This application is no longer available.}")
    fi

    set -x
    mkdir -p "log/eol/${id}"
    flatpak build-commit-from \
        "${eol_args[@]}" \
        "${gpg_args[@]}" \
        --no-update-summary \
        --src-repo=repo \
        --verbose \
        repo \
        "${ref}" \
        2>&1 | tee "log/eol/${id}/${arch}.txt"

clean:
    rm -rf build log

distclean: clean
    rm -rf .flatpak-builder repo

ostree-log:
    #!/usr/bin/env bash
    set -e
    ostree --repo=repo refs | while read ref
    do
        ostree --repo=repo log "${ref}"
    done

ostree-show:
    #!/usr/bin/env bash
    set -e
    ostree --repo=repo refs | while read ref
    do
        ostree --repo=repo show "${ref}"
    done

ubuntu-deps:
    sudo apt-get install --yes \
        flatpak \
        flatpak-builder
