FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:latest

ENV RUBY_PLATFORM="aarch64-linux" \
  CARGO_BUILD_TARGET="aarch64-unknown-linux-gnu" \
  CARGO_HOME="/opt/rust" \
  RUSTUP_HOME="/opt/rust" \
  PATH="/opt/rust/bin:$PATH"

COPY rustup.sh /
RUN /rustup.sh "$CARGO_BUILD_TARGET"

COPY baseruby.sh /
RUN /baseruby.sh

COPY xrubies.sh /
RUN /xrubies.sh "$RUBY_PLATFORM"