#!/bin/bash

set -x
set -eo pipefail

# shellcheck disable=SC1091
source /lib.sh

main() {
    local version=3.23.2

    if ! command -v curl &> /dev/null; then
        install_packages curl
    fi

    local td
    td="$(mktemp -d)"

    pushd "${td}"

    curl --retry 3 -sSfL "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-linux-x86_64.sh" -o cmake.sh
    mkdir -p /opt/cmake
    sh cmake.sh --skip-license --prefix=/opt/cmake
    /opt/cmake/bin/cmake --version

    popd

    rm -rf "${td}"
    rm -rf /var/lib/apt/lists/*
    rm "${0}"
}

main "${@}"