#!/bin/bash

set -xeo pipefail

STABLE_RUBY_VERSIONS=("3.1.2" "3.0.4" "2.7.6")

install_packages() {
  if grep -i ubuntu /etc/os-release; then
    apt-get update -qq
    apt-get install --assume-yes --no-install-recommends autoconf bison build-essential libssl-dev libyaml-dev libreadline6-dev zlib1g-dev libncurses5-dev libffi-dev libgdbm-dev libdb-dev
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives
  elif grep -i centos /etc/os-release; then
    yum install -y gcc-6 bzip2 openssl-devel libyaml-devel libffi-devel readline-devel zlib-devel gdbm-devel ncurses-devel
    yum clean all
  fi
}

install_ruby_build() {
  local td
  td="$(mktemp -d)"
  builtin pushd "$td"
  git clone --depth 1 https://github.com/rbenv/ruby-build.git
  PREFIX=/usr/local ./ruby-build/install.sh
  popd
  rm -rf "$td"
}

main() {
  echo "gem: --no-ri --no-rdoc" >> ~/.gemrc

  install_ruby_build
  install_packages

  export RUBY_CONFIGURE_OPTS="--disable-install-doc --disable-install-rdoc"
  # shellcheck disable=SC2155
  export MAKEOPTS="-j$(nproc)"

  for version in "${STABLE_RUBY_VERSIONS[@]}"; do
    ruby-build "$version" "/opt/rubies/$version" --verbose
  done

  rm "${0}"
}

main "${@}"
