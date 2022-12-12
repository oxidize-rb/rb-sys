#!/bin/sh

main() {
  rb_sys_dock_bash="$(cat \
<<'EOF'
# Codespaces bash prompt theme
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
    echo "RUSTUP_DEFAULT_TOOLCHAIN="$RUSTUP_DEFAULT_TOOLCHAIN"
    echo "RUSTUP_HOME=$RUSTUP_HOME"
    echo "CARGO_HOME=$CARGO_HOME"
    echo "RUSTFLAGS=$RUSTFLAGS"
    echo "RUBY_CC_VERSION=$RUBY_CC_VERSION"
}

__set_command_history() {
    if [ -d /tmp/commandhistory ]; then
        export HISTFILE=/tmp/commandhistory/.bash_history
        export PROMPT_COMMAND='history -a'
    fi
}

__first_notice() {
    local lightblue="\033[0;34m"
    local removecolor="\033[0m"

    echo "${lightblue}Welcome to the rb-sys-dock container!${removecolor}"
    echo
    echo "To see the environment variables that are set, run:\n  $ rb-sys-env"
    echo
    echo "Here are some steps to help you get started:"
    [[ -f Gemfile ]] && echo "  0. Run 'bundle install' to install the gems in your Gemfile"
    echo "  1. Run 'rake native:$RUBY_TARGET' to build the native extension"
    echo
}

if [ "$USER" = "rb-sys-dock" ]; then
    __set_command_history
    __bash_prompt
    __first_notice
fi
EOF
)"

  echo "${rb_sys_dock_bash}" >> /etc/skel/.bashrc
  rm "${0}"
}

main "$@"
