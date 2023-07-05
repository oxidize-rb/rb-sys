# Contributing

So you want to help make this project better? Well, you're in the right place. This project is open source, and we
welcome contributions from everyone. Here is some information about how to develop in this project.

<!-- regenerate TOC with `md-doc CONTRIBUTING.md --min-depth 1` -->
<!-- toc -->

## Table of Contents

1. [Set up your local development environment](#set-up-your-local-development-environment) 1.
[Using the VSCode Devcontainer](#using-the-vscode-devcontainer) 1. [Manual way](#manual-way)
<!-- tocstop -->

## Set up your local development environment

### Using the VSCode Devcontainer

This repo contains a [VSCode Devcontainer](https://code.visualstudio.com/docs/containers/devcontainer) configuration to
make contribution easier. Here are the steps to use it:

1. `git clone https://github.com/oxidize-rb/rb-sys`
2. `code ./rb-sys`
3. Click the green `Reopen in Container...` button on the bottom right

### Manual way

1. `git clone https://github.com/oxidize-rb/rb-sys`
2. Make sure you have Ruby installed
3. Install Rust and Cargo with [Rustup](https://rustup.rs/).

## Running benchmarks

To run the benchmarks, make sure your dev environment, then run `cargo bench`.
This will run the `criterion` benchmarks and print the results to the console.

To see see plots of the results, you can open
`target/criterion/report/index.html` in your browser.
