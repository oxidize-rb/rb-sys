# frozen_string_literal: true

namespace :cli do
  desc "Run rb-sys-cli phase 0 (OCI download/extract)"
  task :phase_0 do
    sh "./script/run", "cargo", "run",
      "--release",
      "-p", "rb-sys-cli-phase-0",
      "--",
      "--config", "data/derived/rb-sys-cli.json"
  end

  desc "Run rb-sys-cli phase 1 (codegen + packaging)"
  task :phase_1 do
    cache_dir = ENV.fetch("RB_SYS_BUILD_CACHE_DIR", File.expand_path("~/.cache/rb-sys/cli"))

    sh "./script/run", "cargo", "run",
      "--release",
      "-p", "rb-sys-cli-phase-1",
      "--",
      "all",
      "--toolchains-json", "data/toolchains.json",
      "--config", "data/derived/rb-sys-cli.json",
      "--cache-dir", cache_dir,
      "--derived-dir", "data/derived",
      "--embedded-dir", "crates/rb-sys-cli/src/embedded"
  end

  desc "Prepare all rb-sys-cli derived artifacts (phase 0 + phase 1)"
  task prepare: [:phase_0, :phase_1]

  desc "Build the rb-sys-cli binary"
  task :build do
    sh "./script/run", "cargo", "build", "-p", "rb-sys-cli"
  end

  desc "Build the rb-sys-cli binary in release mode"
  task :build_release do
    sh "./script/run", "cargo", "build", "-p", "rb-sys-cli", "--release"
  end
end
