EXAMPLES = Dir["examples/*"]

def extra_args
  seperator_index = ARGV.index("--")
  seperator_index && ARGV[(seperator_index + 1)..-1]
end

namespace :test do
  desc "Test against all installed Rubies"
  task :rubies do
    ["rb-sys", "rb-sys-tests"].each do |pkg|
      cargo_args = extra_args || ["--features", "link-ruby", "--quiet"]

      cmd = <<~SH
        gem install bundler:2.3.7
        bundle check || bundle install -j3
        cargo test -p #{pkg} #{cargo_args.join(" ")}
      SH

      sh "./script/xruby", "-c", cmd
    end
  end

  namespace :examples do
    task :rust_reverse do
      Dir.chdir("examples/rust_reverse") do
        sh "rake"
      end
    end
  end

  desc "Test all examples against all installed Rubies"
  task examples: ["test:examples:rust_reverse"]
end

desc "Run all tests"
task test: ["test:rubies", "test:examples"]

desc "Pretty the files"
task :fmt do
  sh "cargo fmt"
  sh "standardrb --fix"
  sh "npx prettier --write $(git ls-files '*.yml')"
end
task format: [:fmt]

desc "Lint"
task :lint do
  sh "bundle exec standardrb --format #{ENV.key?("CI") ? "github" : "progress"}"
  sh "cargo fmt --check"
  sh "cargo clippy"
  sh "shellcheck $(git ls-files '*.sh')"
end
