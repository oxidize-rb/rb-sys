EXAMPLES = Dir["examples/*"]

CLEAN = Rake::FileList.new.tap do |list|
  list.include("**/target")
  list.include("**/tmp")
end

def extra_args
  seperator_index = ARGV.index("--")
  seperator_index && ARGV[(seperator_index + 1)..-1] || []
end

def cargo_test_task(name, *args)
  task_name = "cargo:#{name}"

  desc "Run cargo tests for #{name.inspect} against current Ruby"
  task task_name do
    ENV["RUST_TEST_THREADS"] ||= "1"
    default_args = ENV["CI"] || extra_args.include?("--verbose") ? [] : ["--quiet"]
    sh "cargo", "test", *default_args, *extra_args, *args
  end

  task cargo: task_name
end

namespace :test do
  cargo_test_task "rb-sys", "--features", "bindgen-layout-tests"
  cargo_test_task "rb-sys-build"
  cargo_test_task "rb-sys-tests"
  cargo_test_task "rb-sys-env"

  desc "Test against all installed Rubies"
  task :rubies do
    cmd = <<~SH
      gem install bundler:2.3.7 > /dev/null 2>&1
      bundle check || bundle install -j3 > /dev/null 2>&1
      bundle exec rake test:cargo
    SH

    sh "./script/xruby", "-c", cmd
  end

  namespace :examples do
    task :rust_reverse do
      cargo_args = extra_args || []
      envs = [{}, {"ALTERNATE_CONFIG_SCRIPT" => "extconf_bare.rb"}]

      Dir.chdir("examples/rust_reverse") do
        envs.each do |env|
          sh env, "rake", "clean", "compile", "test", *cargo_args
        end
      end
    end
  end

  desc "Test all examples against all installed Rubies"
  task examples: ["test:examples:rust_reverse"]

  desc "Run unit tests for the gem"
  task :gem do
    Dir.chdir("gem") do
      sh "rake"
    end
  end
end

desc "Run all tests"
task test: ["test:cargo", "test:gem", "test:examples"]

desc "Pretty the files"
task :fmt do
  sh "cargo fmt"
  sh "bundle exec standardrb --fix" if RUBY_VERSION >= "2.6.0"
  sh "npx prettier --write $(git ls-files '*.yml')"
  md_files = `git ls-files '*.md'`.split("\n").select { |f| File.exist?(f) }
  sh "npx", "prettier", "--write", "--print-width=120", "--prose-wrap=always", *md_files
end
task format: [:fmt]

desc "Lint"
task :lint do
  sh "bundle exec standardrb --format #{ENV.key?("CI") ? "github" : "progress"}" if RUBY_VERSION >= "2.6.0"
  sh "cargo fmt --check"
  sh "cargo clippy"
  sh "shellcheck $(git ls-files '*.sh')"
end

namespace :data do
  desc "Derive useful data from data/toolchains.json"
  task :derive do
    require "json"

    gen = ->(name, value) { File.write(File.join("data/derived", name), JSON.pretty_generate(value)) }
    toolchains = JSON.parse(File.read("data/toolchains.json"))
    toolchain_info_data_path = "gem/lib/rb_sys/toolchain_info/data.rb"
    toolchain_data = {}
    toolchains["toolchains"].each do |t|
      tc = t.dup
      tc.delete("dockerfile")
      toolchain_data[tc.delete("ruby-platform")] = tc
    end

    File.write toolchain_info_data_path, <<~RUBY
      # frozen_string_literal: true

      # THIS FILE IS AUTO-GENERATED BY `rake data:derive`

      module RbSys
        class ToolchainInfo
          # @private
          DATA = #{toolchain_data.inspect}
        end
      end
    RUBY

    sh "bundle exec standardrb --fix #{toolchain_info_data_path}"

    ruby_to_rust = {}

    toolchains["toolchains"].each do |t|
      raise "no dockerfile" unless File.exist?(t["dockerfile"])
      raise "wrong ruby target" unless File.read(t["dockerfile"]).include?(t["ruby-platform"])

      ruby_to_rust[t["ruby-platform"]] = t["rust-target"]
    end

    github_actions_matrix = toolchains["toolchains"]
      .select { |t| t["supported"] }
      .map { |t| t.slice("ruby-platform", "rust-target") if t["supported"] }

    gen.call("ruby-to-rust.json", ruby_to_rust)
    gen.call("github-actions-matrix.json", {include: github_actions_matrix})
  end
end

namespace :bindings do
  desc "Copy bindings to /tmp/bindings"
  task :generate do
    require "rbconfig"

    c = RbConfig::CONFIG
    version = RbSys::VERSION
    sh "cargo build || env RB_SYS_DEBUG_BUILD=1 cargo build"
    out_dir = "/tmp/bindings/rb-sys-#{version}/#{c["ruby_version"]}/#{c["arch"]}"
    bindings_file = Dir["target/debug/build/rb-sys-*/out/bindings.rs"].max_by { |f| File.mtime(f) }

    abort "No bindings file found" unless bindings_file

    puts "Copying #{bindings_file} to #{out_dir}/bindings.rs"
    FileUtils.mkdir_p(out_dir)
    FileUtils.cp(bindings_file, out_dir)
  end
end

namespace :debug do
  task :mkmf do
    require "tmpdir"
    tmpdir = Dir.mktmpdir

    chdir(tmpdir) do
      touch "testing.c"
      touch "testing.h"
      sh "ruby", "-rmkmf", "-e", "create_makefile('testing')"
      puts File.read("Makefile")
    end

    rm_rf(tmpdir)
  end
end

desc "Clean up"
task :clean do
  CLEAN.each { |f| rm_rf(f) }
end

task default: :test
