# rakelib/doctest.rake

require "rake"
require "fileutils"

namespace :doctest do
  desc "Run documentation code examples tests"
  task :run do
    test_dir = "tmp/doctests"
    src_dir = File.join(test_dir, "src")
    puts "--- Created temporary directory for doctests at #{test_dir} ---"

    files_to_check = ENV["FILE"] ? [ENV["FILE"]] : Dir.glob("docsite/docs/**/*.{mdx,md}")
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
      example_to_run = ENV["EXAMPLE"] ? ENV["EXAMPLE"].to_i - 1 : nil

      rust_code_blocks.each_with_index do |rust_code, index|
        next if example_to_run && index != example_to_run
        slug = file.gsub(/[^a-zA-Z0-9_]/, "_")
        rust_file_name = "#{slug}_#{index}.rs"
        rust_files << rust_file_name
        # Remove #[magnus::init] to avoid symbol conflicts in doctests
        cleaned_code = rust_code.gsub(/^#\[magnus::init\]\s*\n/, "")
        File.write(File.join(src_dir, rust_file_name), cleaned_code)
      end
    end

    if rust_files.any?
      puts "--- Compiling all Rust examples ---"
      lib_rs_content = "#![allow(dead_code)]\n#![deny(clippy::unwrap_used)]\n#![deny(clippy::expect_used)]\n#![deny(clippy::indexing_slicing)]\n\n" + rust_files.map do |file|
        module_name = File.basename(file, ".rs")
        # Check if this file uses rb_sys_test_helpers
        file_content = File.read(File.join(src_dir, file))
        if file_content.include?("rb_sys_test_helpers")
          # Put test helpers in a test module
          %(#[cfg(test)]\n#[path="#{file}"] mod #{module_name};)
        else
          %(#[path="#{file}"] mod #{module_name};)
        end
      end.join("\n")
      File.write(File.join(src_dir, "lib.rs"), lib_rs_content)

      cargo_toml_content = <<~EOF
        [package]
        name = "doctests"
        version = "0.1.0"
        edition = "2021"
        
        [workspace]
        
        [dependencies]
        magnus = { version = "0.8", features = ["rb-sys"] }
        serde_magnus = "0.10"
        rb-sys = "0.9"
        unicode-segmentation = "1.1.0"
        serde = { version = "1.0", features = ["derive"] }
        serde_json = "1.0"
        regex = "1.10.2"
        rayon = "1.9.0"
        csv = "1.3.0"
        sha2 = "0.10.8"
        hex = "0.4.3"
        image = "0.25.9"
        once_cell = "1.19.0"
        log = "0.4.21"
        env_logger = "0.11.0"
        criterion = "0.5.1" # Latest is 0.5.1, so keep it.
        url = "2.5.0"
        lz4_flex = "0.12.0"
        
        [build-dependencies]
        rb-sys-env = { path = "../../crates/rb-sys-env" }
        
        [dev-dependencies]
        rb-sys-test-helpers = { path = "../../crates/rb-sys-test-helpers" }
        proptest = "1.4.0"
        
        [patch.crates-io]
        rb-sys = { path = "../../crates/rb-sys" }
      EOF
      File.write(File.join(test_dir, "Cargo.toml"), cargo_toml_content)

      # Create build.rs to handle macOS framework linking
      build_rs_content = <<~EOF
        use std::env;
        
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let target = env::var("TARGET")?;
            
            // Link SystemConfiguration framework on macOS for reqwest
            if target.contains("apple") {
                println!("cargo:rustc-link-lib=framework=SystemConfiguration");
                println!("cargo:rustc-link-lib=framework=CoreFoundation");
                println!("cargo:rustc-link-lib=framework=Security");
            }
            
            // Use rb-sys-env to set up Ruby linking
            rb_sys_env::activate()?;
            Ok(())
        }
      EOF
      File.write(File.join(test_dir, "build.rs"), build_rs_content)

      # Explicitly create target directories to avoid "No such file or directory" errors
      FileUtils.mkdir_p(File.join(test_dir, "target", "debug", "deps"))

      # Set CARGO_TARGET_DIR to a known, existing path
      cargo_target_dir = File.join(Dir.pwd, test_dir, "cargo_target") # Make it absolute from the current working directory
      FileUtils.mkdir_p(cargo_target_dir)
      ENV["CARGO_TARGET_DIR"] = cargo_target_dir

      Dir.chdir(test_dir) do
        sh("cargo", "check", "--all-targets")
        puts "✅ All Rust examples compiled successfully."

        # Run clippy to check for lints
        puts "--- Running clippy on Rust examples ---"

        # Run clippy with strict checks
        sh("cargo", "clippy", "--all-targets", "--",
          "-A", "dead_code",
          "-A", "clippy::redundant_field_names",
          "-D", "clippy::unwrap_used",
          "-D", "clippy::expect_used",
          "-D", "clippy::indexing_slicing")
        puts "✅ All Rust examples pass clippy checks."
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
      example_to_run = ENV["EXAMPLE"] ? ENV["EXAMPLE"].to_i - 1 : nil

      ruby_code_blocks.each_with_index do |ruby_code, index|
        next if example_to_run && index != example_to_run

        print "  - Checking example ##{index + 1}... "
        temp_ruby_file = File.join(test_dir, "temp_ruby_#{index}.rb")
        File.write(temp_ruby_file, ruby_code)

        sh("ruby", "-c", temp_ruby_file)
      end
    end

    puts "✅ All documentation code examples are valid!"
  end
end
