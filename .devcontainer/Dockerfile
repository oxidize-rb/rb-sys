ARG VARIANT
FROM mcr.microsoft.com/vscode/devcontainers/rust:${VARIANT}

ENV CHRUBY_VERSION="0.3.9" \
    RUBY_INSTALL_VERSION="0.8.3" \
    BUILD_LIST="build-essential" \
    DEBIAN_FRONTEND="noninteractive" \
    RUBY_VERSIONS="2.6.9 2.7.5 3.1.1 3.2.0-preview1" 

USER root

RUN set -ex; \
    apt-get update && \
    apt-get install -y --no-install-recommends $BUILD_LIST && \

    # postmodern gpg
    cd /tmp; \
    wget https://raw.github.com/postmodern/postmodern.github.io/main/postmodern.asc; \
    gpg --import postmodern.asc; \
    rm postmodern.asc; \

    # Install chruby
    mkdir /tmp/chruby-build; \
    cd /tmp/chruby-build; \
    wget -O chruby-$CHRUBY_VERSION.tar.gz https://github.com/postmodern/chruby/archive/v$CHRUBY_VERSION.tar.gz; \
    wget https://raw.github.com/postmodern/chruby/master/pkg/chruby-$CHRUBY_VERSION.tar.gz.asc; \
    gpg --verify chruby-$CHRUBY_VERSION.tar.gz.asc chruby-$CHRUBY_VERSION.tar.gz; \
    tar -xzvf chruby-$CHRUBY_VERSION.tar.gz; \
    cd chruby-$CHRUBY_VERSION/; \
    sudo bash -c "./scripts/setup.sh"; \
    echo "source /usr/local/share/chruby/chruby.sh" >> "$HOME/.zshrc"; \
    echo "chruby 3.1.1" >> "$HOME/.zshrc"; \
    echo "source /usr/local/share/chruby/chruby.sh" >> "$HOME/.bashrc"; \
    echo "chruby 3.1.1" >> "$HOME/.bashrc"; \
    cd /; \

    # Install ruby-install
    mkdir /tmp/ruby-install-build; \
    cd /tmp/ruby-install-build; \
    wget -O ruby-install-${RUBY_INSTALL_VERSION}.tar.gz https://github.com/postmodern/ruby-install/archive/v${RUBY_INSTALL_VERSION}.tar.gz; \
    wget https://raw.github.com/postmodern/ruby-install/master/pkg/ruby-install-${RUBY_INSTALL_VERSION}.tar.gz.asc; \
    gpg --verify ruby-install-${RUBY_INSTALL_VERSION}.tar.gz.asc ruby-install-${RUBY_INSTALL_VERSION}.tar.gz; \
    tar -xzvf ruby-install-${RUBY_INSTALL_VERSION}.tar.gz; \
    cd ruby-install-${RUBY_INSTALL_VERSION}/; \
    make install; \
    cd /; \

    # Install ruby versions
    echo 'gem: --no-document' >> $HOME/.gemrc; \
    for ver in $RUBY_VERSIONS; do \
      # static
      ruby-install -c -j$(nproc) --install-dir "/opt/rubies/$ver-static" ruby "$ver" -- --disable-install-rdoc; \
      # dynamic
      ruby-install -c -j$(nproc) --install-dir "/opt/rubies/$ver" ruby "$ver" -- --disable-install-rdoc --enable-shared; \
    done; \

    # Cleanup
    apt-get clean && \
    rm -rf /tmp/* /var/lib/apt/lists/* /var/tmp/* /ruby-install* /chruby* /usr/local/src/ruby*;
  
  RUN set -ex; \
      cargo install cargo-make; \
      mkdir -p $HOME/.config/cargo-make; \
      echo 'log_level = "error"' >> $HOME/.config/cargo-make/config.toml;

  COPY welcome-message.txt "/usr/local/share/welcome-message.txt"
  RUN echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.zshrc; \
      echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.bashrc;