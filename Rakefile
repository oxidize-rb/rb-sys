EXAMPLES = Dir["examples/*"]

def extra_args
  seperator_index = ARGV.index("--")
  seperator_index && ARGV[(seperator_index + 1)..-1]
end

namespace :test do
  desc "Test against all installed Rubies"
  task :rubies do
    cargo_args = extra_args || ["--quiet"]

    sh "./script/xruby", "-c", "cargo test #{cargo_args.join(" ")}"
  end

  namespace :examples do
    task :rust_reverse do
      Dir.chdir("examples/rust_reverse") do
        sh "rake test"
      end
    end
  end

  desc "Test all examples against all installed Rubies"
  task examples: ["test:examples:rust_reverse"]
end