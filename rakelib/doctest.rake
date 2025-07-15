# rakelib/doctest.rake

require "rake"
require "fileutils"

namespace :doctest do
  desc "Run documentation code examples tests"
  task :run do
    test_dir = "tmp/doctests"
    src_dir = File.join(test_dir, "src")
    FileUtils.rm_rf(test_dir)
    FileUtils.mkdir_p(src_dir)
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
      lib_rs_content = "#![deny(clippy::unwrap_used)]\n#![deny(clippy::expect_used)]\n\n" + rust_files.map do |file|
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
        image = "0.24"
        once_cell = "1.17"
        log = "0.4"
        env_logger = "0.9"
        criterion = "0.5"
        url = "2.5"
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

      Dir.chdir(test_dir) do
        sh("cargo", "check", "--all-targets")
        puts "✅ All Rust examples compiled successfully."

        # Run clippy to check for lints
        puts "--- Running clippy on Rust examples ---"

        # For doctests, we allow warnings since many functions/variables are unused in examples
        # We also allow some clippy lints that are too strict for documentation
        sh("cargo", "clippy", "--all-targets", "--",
          "-A", "dead_code",
          "-A", "unused_variables",
          "-A", "unused_imports",
          "-A", "unused_mut",
          "-A", "clippy::needless_range_loop",
          "-A", "clippy::redundant_field_names",
          "-A", "clippy::manual_c_str_literals",
          "-D", "clippy::unwrap_used",
          "-D", "clippy::expect_used")
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

    # --- Cleanup ---
    puts "--- Cleaning up temporary directory ---"
    FileUtils.rm_rf(test_dir)

    puts "✅ All documentation code examples are valid!"
  end
end
