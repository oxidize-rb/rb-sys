namespace :fuzz do
  desc "Run fuzz tests for a target"
  task :run do
    target = ENV.fetch("TARGET") do
      candidates = `cargo fuzz list`.split("\n")
      abort "Must specify TARGET environment variable. Candidates: #{candidates.join(", ")}"
    end

    sh "cargo", "+nightly", "fuzz", "run", "--sanitizer=none", target
  end
end
