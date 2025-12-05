# Build repo
repo:
    #!/usr/bin/env bash
    set -e

    # Build all apps
    ls -1 app | while read id
    do
        just build ${id}
    done

    cat end-of-life.txt | grep -v '^#' | while read eol
    do
        id="$(echo "${eol}" | cut -d '=' -f 1)" 
        rebase="$(echo "${eol}" | cut -d '=' -f 2)" 
        just eol ${id} ${rebase}
    done

    # Generate update information and appstream data
    set -x
    flatpak \
        build-update-repo \
        --gpg-sign="${DEBEMAIL}" \
        --generate-static-deltas \
        --prune \
        repo

# Build app with specified ID
build id:
    #!/usr/bin/env bash
    set -e
    arch="$(flatpak --default-arch)"
    set -x
    mkdir -p "log/app/{{id}}"
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
        "build/app/{{id}}/${arch}" \
        "app/{{id}}/{{id}}.json" \
        2>&1 | tee "log/app/{{id}}/${arch}.txt"

# EOL app with specified id and rebase id
eol id rebase:
    #!/usr/bin/env bash
    set -e
    arch="$(flatpak --default-arch)"
    ref="app/{{id}}/${arch}/master"
    if ostree --repo=repo show "${ref}"
    then
        current_rebase="$(ostree --repo=repo show --print-metadata-key=ostree.endoflife-rebase "${ref}")"
        if [ "${current_rebase}" == "'app/{{rebase}}/${arch}/master'" ]
        then
            echo "{{id}} already rebased to {{rebase}}"
        else
            set -x
            mkdir -p "log/eol/{{id}}"
            flatpak build-commit-from \
                --end-of-life="Application has been renamed to {{rebase}}" \
                --end-of-life-rebase="{{id}}={{rebase}}" \
                --gpg-sign="${DEBEMAIL}" \
                --no-update-summary \
                --src-repo=repo \
                --verbose \
                repo \
                "${ref}" \
                2>&1 | tee "log/eol/{{id}}/${arch}.txt"
        fi
    else
        echo "{{id}} not found, will not try to rebase to {{rebase}}"
    fi

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
