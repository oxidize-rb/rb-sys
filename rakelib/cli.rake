# frozen_string_literal: true

namespace :cli do
  desc "Prepare all rb-sys-cli artifacts (fetch, transform, bundle)"
  task :prepare => [:phase_0, :phase_1, :phase_2]

  desc "Run rb-sys-cli phase 0 (fetch assets for all platforms)"
  task :phase_0 do
    sh "./script/run", "cargo", "run",
      "--release",
      "-p", "rb-sys-cli-phase-0"
  end

  desc "Run rb-sys-cli phase 1 (transform and enhance)"
  task :phase_1 do
    sh "./script/run", "cargo", "run",
      "--release",
      "-p", "rb-sys-cli-phase-1"
  end

  desc "Run rb-sys-cli phase 2 (bundle into tar.xz)"
  task :phase_2 do
    sh "./script/run", "cargo", "run",
      "--release",
      "-p", "rb-sys-cli-phase-2"
  end

  desc "Clean all generated assets and staging"
  task :clean do
    rm_rf "data/staging"
    rm_rf "tmp/cache"
    rm_rf "data/derived/phase_0_lock.toml"
    rm_rf "data/derived/runtime_manifest.json"
    rm_rf "crates/rb-sys-cli/src/embedded/assets.tar.xz"
    rm_rf "crates/rb-sys-cli/src/embedded/runtime_manifest.json"
  end

  desc "Build the rb-sys-cli binary"
  task :build do
    sh "./script/run", "cargo", "build", "-p", "rb-sys-cli"
  end

  desc "Build the rb-sys-cli binary in release mode"
  task :build_release do
    sh "./script/run", "cargo", "build", "-p", "rb-sys-cli", "--release"
  end
end
