#!/bin/sh

main() {
  rb_sys_dock_bash="$(cat \
<<'EOF'
export RB_SYS_DOCK_TMPDIR="/tmp/rb-sys-dock"

__bash_prompt() {
    local ruby_platform='`export XIT=$? \
        && echo -n "\[\033[0;32m\]${RUBY_TARGET} " \
        && [ "$XIT" -ne "0" ] && echo -n "\[\033[1;31m\]➜" || echo -n "\[\033[0m\]➜"`'
    local gitbranch='`\
        export BRANCH=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null); \
        if [ "${BRANCH}" != "" ]; then \
            echo -n "\[\033[0;36m\](\[\033[1;31m\]${BRANCH}" \
            && if git ls-files --error-unmatch -m --directory --no-empty-directory -o --exclude-standard ":/*" > /dev/null 2>&1; then \
                    echo -n " \[\033[1;33m\]✗"; \
            fi \
            && echo -n "\[\033[0;36m\]) "; \
        fi`'
    local lightblue='\[\033[1;34m\]'
    local removecolor='\[\033[0m\]'
    PS1="${ruby_platform} ${lightblue}\W ${gitbranch}${removecolor}\$ "


    unset -f __bash_prompt
}

rb-sys-env() {
    echo "RUBY_TARGET=$RUBY_TARGET"
    echo "RUST_TARGET=$RUST_TARGET"
    echo "RUSTUP_DEFAULT_TOOLCHAIN=$RUSTUP_DEFAULT_TOOLCHAIN"
    echo "RUSTUP_HOME=$RUSTUP_HOME"
    echo "CARGO_HOME=$CARGO_HOME"
    echo "RUSTFLAGS=$RUSTFLAGS"
    echo "RUBY_CC_VERSION=$RUBY_CC_VERSION"
    echo "BUNDLE_PATH=$BUNDLE_PATH"
}

__set_command_history() {
    if [ -d "$RB_SYS_DOCK_TMPDIR/commandhistory" ]; then
        export HISTFILE="$RB_SYS_DOCK_TMPDIR/commandhistory/.bash_history"
        export PROMPT_COMMAND='history -a'
    fi

    unset -f __set_command_history
}

__set_bundle_path() {
    if [ -d "$RB_SYS_DOCK_TMPDIR/bundle" ]; then
        export BUNDLE_PATH="$RB_SYS_DOCK_TMPDIR/bundle"
    fi

    unset -f __set_bundle_path
}

__first_notice() {
    local lightblue="\033[0;34m"
    local removecolor="\033[0m"

    echo "${lightblue}Welcome to the rb-sys-dock container!${removecolor}"
    echo
    echo "To see the environment variables that are set, run:"
    echo "  $ rb-sys-env"
    echo
    echo "Here are some steps to help you get started:"
    [[ -f Gemfile ]] && echo "  0. Run 'bundle install' to install the gems in your Gemfile"
    echo "  1. Run 'rake native:$RUBY_TARGET' to build the native extension"
    echo
}

rg() {
  if [ ! -f /usr/local/bin/rg ]; then
    if [ "$(uname -m)" != "x86_64" ]; then
      echo "ripgrep is not installed"
      exit 1
    fi

    echo "Installing ripgrep..."

    # download from github releases, untar, and install to tmpdir
    local rg_version="13.0.0"
    local rg_sha256="4ef156371199b3ddac1bf584e0e52b1828279af82e4ea864b4d9c816adb5db40"
    local rg_tmpdir="$(mktemp -d)"
    local url="https://github.com/BurntSushi/ripgrep/releases/download/$rg_version/ripgrep-$rg_version-x86_64-unknown-linux-musl.tar.gz"
    curl -sL "$url" | tar -xz -C "$rg_tmpdir" --strip-components=1
    sha256sum "$rg_tmpdir/rg" | grep "$rg_sha256" > /dev/null

    # move to /usr/local/bin
    sudo mv "$rg_tmpdir/rg" /usr/local/bin
    /usr/local/bin/rg "$@"

  fi

  unset -f rg
}

nvim() {

  if [ ! -f /usr/local/bin/vim ]; then
    if [ "$(uname -m)" != "x86_64" ]; then
      echo "vim is not installed"
      exit 1
    fi

    echo "Installing nvim..."

    # download from github releases, untar, and install to tmpdir
    local vim_version="v0.8.1"
    local vim_sha256="2cabf3973936a0fd25d11b95e23bb4cae686bb916de7a0cb24cd16fe6764a0f4"
    local vim_tmpdir="$(mktemp -d)"
    local url="https://github.com/neovim/neovim/releases/download/$vim_version/nvim-linux64.tar.gz"

    curl -sL "$url" | tar -xz -C "$vim_tmpdir" --strip-components=1
    sha256sum "$vim_tmpdir/bin/nvim" | grep "$vim_sha256" > /dev/null

    sudo mv "$vim_tmpdir" /usr/local/nvim
    export PATH="/usr/local/nvim/bin:$PATH"
    sudo ln -s /usr/local/nvim/bin/nvim /usr/local/bin/vim
    unset -f vim
    unset -f nvim
    /usr/local/nvim/bin/nvim "$@"
  fi
}

vim() {
  nvim "$@"
}

if [ "$USER" = "rb-sys-dock" ]; then
    __set_command_history
    __set_bundle_path
    __bash_prompt
    __first_notice
fi
EOF
)"

  echo "${rb_sys_dock_bash}" >> /etc/skel/.bashrc
  rm "${0}"
}

main "$@"
