# rakelib/doctest.rake

require 'rake'
require 'fileutils'
require 'open3'

namespace :doctest do
  desc "Run documentation code examples tests"
  task :run do
    test_dir = "tmp/doctests"
    src_dir = File.join(test_dir, "src")
    FileUtils.rm_rf(test_dir)
    FileUtils.mkdir_p(src_dir)
    puts "--- Created temporary directory for doctests at #{test_dir} ---"

    # Set Ruby environment variables for the build process
    # ruby_root = ENV['RUBY_ROOT'] || '/Users/ianks/.rubies/ruby-3.4.1'
    # ruby_version = ENV['RUBY_VERSION'] || '3.4.1'
    # ENV['RUBY_ROOT'] ||= ruby_root
    # ENV['RUBY_VERSION'] ||= ruby_version
    # ENV['PATH'] = "#{ruby_root}/bin:#{ENV['PATH']}"

    files_to_check = ENV['FILE'] ? [ENV['FILE']] : Dir.glob("docsite/docs/**/*.{mdx,md}")
    rust_files = []

    # --- Rust Doctests ---
    puts "--- Extracting Rust code examples ---"
    files_to_check.each do |file|
      file_content = File.read(file)
      chunks = file_content.split("```")
      rust_code_blocks = []

      chunks.each_with_index do |chunk, i|
        next if i.even?
        lines = chunk.split("\n")
        lang = lines.first.to_s.strip

        if lang.start_with?("rust") && !lang.include?("ignore")
          code = lines.slice(1..-1)&.join("\n")
          rust_code_blocks << code if code && !code.strip.empty?
        end
      end

      next if rust_code_blocks.empty?

      puts "Processing Rust in #{file}"
      example_to_run = ENV['EXAMPLE'] ? ENV['EXAMPLE'].to_i - 1 : nil

      rust_code_blocks.each_with_index do |rust_code, index|
        next if example_to_run && index != example_to_run
        slug = file.gsub(/[^a-zA-Z0-9_]/, '_')
        rust_file_name = "#{slug}_#{index}.rs"
        rust_files << rust_file_name
        File.write(File.join(src_dir, rust_file_name), rust_code)
      end
    end

    if rust_files.any?
      puts "--- Compiling all Rust examples ---"
      lib_rs_content = rust_files.map do |file|
        module_name = File.basename(file, ".rs")
        %Q(#[path="#{file}"] mod #{module_name};)
      end.join("\n")
      File.write(File.join(src_dir, "lib.rs"), lib_rs_content)

      cargo_toml_content = <<~EOF
[package]
name = "doctests"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
magnus = { version = "0.6", features = ["rb-sys"] }
rb-sys = "0.9"
unicode-segmentation = "1.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.5"
rayon = "1.5"
csv = "1.1"
sha2 = "0.10"
hex = "0.4"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
image = "0.24"
once_cell = "1.17"
log = "0.4"
env_logger = "0.9"
criterion = "0.5"
url = "2.5"
wasmtime = "20.0"
lz4_flex = "0.11"

[build-dependencies]
rb-sys-env = { path = "../../crates/rb-sys-env" }

[dev-dependencies]
rb-sys-test-helpers = { path = "../../crates/rb-sys-test-helpers" }
proptest = "1.0"

[patch.crates-io]
rb-sys = { path = "../../crates/rb-sys" }
      EOF
      File.write(File.join(test_dir, "Cargo.toml"), cargo_toml_content)

      Dir.chdir(test_dir) do
        _stdout, stderr, status = Open3.capture3("cargo", "check", "--tests", "--quiet")

        if status.success?
          puts "✅ All Rust examples compiled successfully."
        else
          puts "❌ Rust compilation failed:"
          puts stderr
          exit 1
        end
      end
    end

    # --- Ruby Doctests ---
    puts "--- Extracting and testing Ruby code examples ---"
    files_to_check.each do |file|
      file_content = File.read(file)
      chunks = file_content.split("```")
      ruby_code_blocks = []

      chunks.each_with_index do |chunk, i|
        next if i.even?
        lines = chunk.split("\n")
        lang = lines.first.to_s.strip

        if lang.start_with?("ruby") && !lang.include?("ignore")
          code = lines.slice(1..-1)&.join("\n")
          ruby_code_blocks << code if code && !code.strip.empty?
        end
      end

      next if ruby_code_blocks.empty?

      puts "Processing Ruby in #{file}"
      example_to_run = ENV['EXAMPLE'] ? ENV['EXAMPLE'].to_i - 1 : nil

      ruby_code_blocks.each_with_index do |ruby_code, index|
        next if example_to_run && index != example_to_run

        print "  - Checking example ##{index + 1}... "
        temp_ruby_file = File.join(test_dir, "temp_ruby_#{index}.rb")
        File.write(temp_ruby_file, ruby_code)

        _stdout, stderr, status = Open3.capture3("ruby", "-c", temp_ruby_file)

        if status.success?
          puts "✅"
        else
          puts "❌"
          puts "Ruby in #{file} (example ##{index + 1}) has syntax errors:"
          puts stderr
          exit 1
        end
      end
    end

    # --- Cleanup ---
    puts "--- Cleaning up temporary directory ---"
    FileUtils.rm_rf(test_dir)

    puts "✅ All documentation code examples are valid!"
  end
end
