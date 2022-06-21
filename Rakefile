EXAMPLES = Dir["examples/*"]

def extra_args
  seperator_index = ARGV.index("--")
  seperator_index && ARGV[(seperator_index + 1)..-1]
end

namespace :test do
  desc "Run cargo test against current Ruby"
  task :cargo do
    cargo_args = extra_args || ["--quiet", "--workspace", "--exclude", "rust-reverse"]
    sh "cargo", "test", *cargo_args
  end

  desc "Test against all installed Rubies"
  task :rubies do
    cargo_args = extra_args || ["--quiet"]

    cmd = <<~SH
      gem install bundler:2.3.7
      bundle check || bundle install -j3
      cargo test #{cargo_args.join(" ")}
    SH

    sh "./script/xruby", "-c", cmd
  end

  namespace :examples do
    task :rust_reverse do
      Dir.chdir("examples/rust_reverse") do
        sh "rake clean compile test"
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
  sh "standardrb --fix"
  sh "npx prettier --write $(git ls-files '*.yml')"
  md_files = `git ls-files '*.md'`.split("\n").select { |f| File.exist?(f) }
  sh "npx", "prettier", "--write", "--print-width=120", "--prose-wrap=always", *md_files
end
task format: [:fmt]

desc "Lint"
task :lint do
  sh "bundle exec standardrb --format #{ENV.key?("CI") ? "github" : "progress"}"
  sh "cargo fmt --check"
  sh "cargo clippy"
  sh "shellcheck $(git ls-files '*.sh')"
end

desc "Bump the gem version"
task :bump do
  printf "What is the new version?: "
  new_version = $stdin.gets.chomp
  sh "fastmod", "--extensions=toml", "^version = \".*\"", "version = #{new_version.inspect}"
  sh "fastmod", "--extensions=rb", "^  VERSION = \".*\"", "  VERSION = #{new_version.inspect}"
  sh "cargo check"
  sh "bundle"
end

desc "Publish the crates and gems"
task :publish do
  Dir.chdir("gem") do
    sh "bundle exec rake release"
  end

  ["crates/rb-sys-build", "crates/rb-sys", "crates/rb-allocator"].each do |dir|
    Dir.chdir(dir) do
      sh "cargo publish || true"
      sleep 5
    end
  end
end
