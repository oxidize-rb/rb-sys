#!/bin/bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
source /lib.sh

build_from_source() {
  local clang_version
  clang_version="12.0.1"

  local install_dir
  install_dir="/usr/lib/llvm"

  local td
  td="$(mktemp -d)"

  local tarfile
  tarfile="llvmorg-$clang_version.tar.gz"

  local shared_llvm_flags
  shared_llvm_flags="-DLLVM_INCLUDE_BENCHMARKS=Off -DLLVM_INCLUDE_EXAMPLES=Off -DLLVM_INCLUDE_TESTS=Off -DCMAKE_BUILD_TYPE=Release -DLLVM_TARGETS_TO_BUILD=X86 -DLLVM_PARALLEL_LINK_JOBS=1 -DCMAKE_INSTALL_PREFIX=$install_dir -DCMAKE_PREFIX_PATH=$install_dir"

  builtin pushd "${td}"
  curl -L -o "$tarfile" https://github.com/llvm/llvm-project/archive/refs/tags/llvmorg-$clang_version.tar.gz
  tar -xzf "$tarfile" --strip-components=1

  builtin cd llvm
  mkdir -p build-release
  builtin cd build-release
  cmake .. -G 'Unix Makefiles' $shared_llvm_flags -DLLVM_ENABLE_LIBXML2=OFF
  make install
  builtin cd ../..

  # LLD
  builtin cd lld
  mkdir -p build-release
  builtin cd build-release
  cmake .. -G 'Unix Makefiles' $shared_llvm_flags -DCMAKE_CXX_STANDARD=17
  make install
  builtin cd ../..

  # Clang
  builtin cd clang
  mkdir -p build-release
  builtin cd build-release
  cmake .. -G 'Unix Makefiles' $shared_llvm_flags
  make install
  builtin cd ../..

  builtin popd
  rm -rf "${td}"
}

install_rpm() {
  wget https://vault.centos.org/centos/8/AppStream/x86_64/os/Packages/clang-libs-12.0.1-4.module_el8.5.0+1025+93159d6c.i686.rpm \
    https://vault.centos.org/centos/8/AppStream/x86_64/os/Packages/llvm-libs-12.0.1-2.module_el8.5.0+918+ed335b90.i686.rpm \
    https://vault.centos.org/centos/8/BaseOS/x86_64/os/Packages/ncurses-libs-6.1-9.20180224.el8.i686.rpm
  rpm -Uvh --nodeps *.rpm
  ln -s /usr/lib/libtinfo.so.6 /usr/lib/libtinfo.so.5
  rm *.rpm
}

main() {
  if command -v rpm > /dev/null; then
    install_rpm
  else
    build_from_source
  fi

  rm "${0}"
}

main "${@}"
