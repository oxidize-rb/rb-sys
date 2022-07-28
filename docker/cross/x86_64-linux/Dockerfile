FROM ghcr.io/cross-rs/x86_64-unknown-linux-gnu:latest

ENV RUBY_PLATFORM="x86_64-linux" \
  CARGO_BUILD_TARGET="x86_64-unknown-linux-gnu" \
  CARGO_HOME="/opt/rust" \
  RUSTUP_HOME="/opt/rust" \
  PATH="/opt/rust/bin:$PATH"

COPY rustup.sh /
RUN /rustup.sh "$CARGO_BUILD_TARGET"

COPY baseruby.sh /
RUN /baseruby.sh

COPY xrubies.sh /
RUN /xrubies.sh "$RUBY_PLATFORM"

COPY lib.sh /
RUN bash -c "source /lib.sh && install_packages libclang-6.0-dev"
ENV LIBCLANG_PATH="/usr/lib/llvm-6.0/lib" \
  BINDGEN_EXTRA_CLANG_ARGS_x86_64_unknown_linux_gnu="-I/usr/lib/llvm-6.0/lib/clang/6.0.0/include"