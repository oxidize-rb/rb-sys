RUBIES = []
RUBIES << Dir["#{ENV["HOME"]}/.asdf/installs/ruby/*/bin/ruby"]
RUBIES << Dir["/opt/rubies/*/bin/ruby"] if RUBIES.empty?
RUBIES << Dir["#{ENV["HOME"]}/.rbenv/versions/*/bin/ruby"] if RUBIES.empty?
RUBIES.flatten!

RUBY_VERSIONS = RUBIES.map { |f| f.split("/")[-3] }

EXAMPLES = Dir["examples/*"]

def extra_args
  seperator_index = ARGV.index("--")
  seperator_index && ARGV[(seperator_index + 1)..-1]
end

namespace :test do
  RUBIES.each_with_index do |ruby_bin, i|
    version = RUBY_VERSIONS[i]

    desc "Test against Ruby #{version}"
    task version do
      puts "🧪 Testing against Ruby #{version}"
      cargo_args = extra_args || ["--quiet"]

      sh({"RUBY" => ruby_bin}, "cargo", "test", *cargo_args)
    end
  end

  desc "Test against all installed Rubies"
  task rubies: RUBY_VERSIONS

  namespace :examples do
    namespace :simple do
      RUBIES.each_with_index do |ruby_bin, i|
        version = RUBY_VERSIONS[i]

        desc "Test examples/simple against Ruby #{version}"
        task version do
          Dir.chdir("examples/simple") do
            puts "🧪 Testing examples/simple against Ruby #{version}"
            old_ruby = ENV["RUBY"]
            ENV["RUBY"] = ruby_bin
            sh "rake", "examples:build:simple"
            bundle = File.join(Dir.pwd, "target/release/rust_ruby_example.#{RbConfig::CONFIG["DLEXT"]}")
            sh ruby_bin, "-r", bundle, "-e", "RustRubyExample.reverse('hello world') == 'dlrow olleh' ? puts('✅ Test passed') :  abort('❌ Test failed')", verbose: false
          ensure
            ENV["RUBY"] = old_ruby
          end
        end
      end
    end

    desc "Test examples/simple against all installed Rubies"
    task simple: RUBY_VERSIONS.map { |v| "test:examples:simple:#{v}" }
  end

  desc "Test all examples against all installed Rubies"
  task examples: ["test:examples:simple"]
end

namespace :examples do
  namespace :build do
    EXAMPLES.each do |example|
      desc "Build #{example}"
      task File.basename(example) do
        Dir.chdir(example) do
          puts "🔨 Building example #{File.basename(example)}"
          cargo_args = extra_args || []
          sh "cargo", "clean", "--release", verbose: false
          sh "cargo", "build", "--quiet", "--release", "--features", "link-ruby", *cargo_args, verbose: false
          built_lib = Dir["target/release/lib*.{dylib,so,dll}"].first
          ext_lib = built_lib.gsub("release/lib", "release/").gsub(/(dylib|so|dll)$/, RbConfig::CONFIG["DLEXT"])
          FileUtils.cp(built_lib, ext_lib)
        end
      end
    end
  end

  desc "Build all examples"
  task build: EXAMPLES.map { |example| "build:#{File.basename(example)}" }
end
