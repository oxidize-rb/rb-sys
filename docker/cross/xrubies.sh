#!/bin/bash

set -x
set -euo pipefail

STABLE_RUBY_VERSIONS=("3.1.2" "3.0.4" "2.7.6")

find_env() {
  env | grep -e "$1" | cut -d '=' -f 2
}

export_if_env_found() {
  if find_env "$2"; then
    # shellcheck disable=SC2155
    export "$1"="$(find_env "$2")"
  fi
}

cross_build_ruby() {
  local host
  host="$1"
  local version
  version="$2"
  local prefix
  prefix="/opt/xrubies/$host/$version"
  local minor
  minor="$(echo "$version" | cut -d '.' -f 1-2)"

  curl --retry 3 -sSfL "https://cache.ruby-lang.org/pub/ruby/$minor/ruby-$version.tar.gz" -o "ruby-$version.tar.gz"
  tar -xzf "ruby-$version.tar.gz"
  pushd "ruby-$version"
  ./autogen.sh || autoreconf --install
  ./configure \
    --prefix="$prefix" \
    --host="$host" \
    --target="$host" \
    --build="$ruby_build" \
    --with-baseruby="/opt/rubies/$version/bin/ruby" \
    --enable-shared \
    --enable-install-static-library \
    --disable-jit-support \
    --disable-install-doc \
    --with-ext=
  make install-cross
}

main() {
  local host
  host="$1"

  export_if_env_found "CC" "^CC_"
  export_if_env_found "CXX" "^CXX_"
  export_if_env_found "LD" "^CARGO_TARGET_.*_LINKER"

  # shellcheck disable=SC2155
  export MAKE="make -j$(nproc)"
  export CFLAGS="-O1 -fno-omit-frame-pointer -fno-fast-math -fstack-protector-strong -s"
  export LDFLAGS="-pipe -s"

  for version in "${STABLE_RUBY_VERSIONS[@]}"; do
    local td
    td="$(mktemp -d)"
    local ruby_build
    ruby_build="$(/opt/rubies/"$version"/bin/ruby -rrbconfig -e 'print RbConfig::CONFIG["build"]')"
    builtin pushd "$td"
    cross_build_ruby "$host" "$version"
    mkdir -p /root/.rake-compiler
    echo "rbconfig-$host-$version: $(find "/opt/rubies/$host/$version" -name rbconfig.rb)" >> /root/.rake-compiler/config.yml
  done

  rm "${0}"
}

main "${@}"
