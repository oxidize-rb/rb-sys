ARG VARIANT
FROM mcr.microsoft.com/vscode/devcontainers/rust:${VARIANT}

ENV CHRUBY_VERSION="0.3.9" \
  BUILD_LIST="build-essential clang libclang-dev openssh-server" \
  DEBIAN_FRONTEND="noninteractive" \
  RUBY_VERSIONS="2.6.9 2.7.6 3.1.2 3.2.0-preview1" 

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

  # Install ruby-build
  mkdir /tmp/ruby-build-build; \
  cd /tmp/ruby-build-build; \
  git clone https://github.com/rbenv/ruby-build.git; \
  PREFIX=/usr/local ./ruby-build/install.sh; \
  cd /; \

  # Install ruby versions
  echo 'gem: --no-document' >> $HOME/.gemrc; \
  for ver in $RUBY_VERSIONS; do \
  RUBY_CONFIGURE_OPTS="--disable-install-rdoc --enable-shared" ruby-build "$ver" "/opt/rubies/$ver"; \
  done; \

  # Cleanup
  apt-get clean && \
  rm -rf /tmp/* /var/lib/apt/lists/* /var/tmp/* /chruby* /usr/local/src/ruby*;

COPY welcome-message.txt "/usr/local/share/welcome-message.txt"
RUN echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.zshrc; \
  echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.bashrc;