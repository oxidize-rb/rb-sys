# frozen_string_literal: true

module RbSys
  # Error is the base class for all errors raised by rb_sys.
  class Error < StandardError; end

  # Raised when a package is not found from the Cargo metadata.
  class PackageNotFoundError < Error
    def initialize(name)
      msg = <<~MSG.chomp.tr("\n", " ")
        Could not find Cargo package metadata for #{@name.inspect}. Please
        check that #{@name.inspect} matches the crate name in your
        Cargo.toml."
      MSG

      super(msg)
    end
  end

  # Raised when Cargo metadata cannot be parsed.
  class CargoMetadataError < Error
    def initialize(err, stderr)
      msg = <<~MSG.chomp.tr("\n", " ")
        Could not infer Rust crate information using `cargo metadata`.

        Original error was:
          #{err.class}: #{err.message}

        Things to check:
          - Check that your ext/*/Cargo.toml at is valid
          - If you are using a workspace, make sure you are the root Cargo.toml exists
          - Make sure `cargo` is installed and in your PATH
      MSG

      if !stderr.empty?
        indented_stderr = stderr.lines.map { |line| "  #{line}" }.join
        msg << "Stderr from `cargo metadata` was:\n#{indented_stderr}"
      end

      super(msg)
    end
  end
end
