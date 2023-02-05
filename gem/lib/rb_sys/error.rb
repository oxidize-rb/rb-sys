# frozen_string_literal: true

module RbSys
  # Error is the base class for all errors raised by rb_sys.
  class Error < StandardError; end

  # PackageNotFoundError is raised when a package is not found from the Cargo metadata.
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

  class CargoMetadataError < Error
    def initialize(err, stderr, manifest_path)
      msg = <<~MSG.chomp.tr("\n", " ")
        Could not parse Cargo metadata. Please check that your Cargo.toml
        is valid. The error was: #{err}

        Looking for this Cargo.toml: #{manifest_path.inspect}

        Stderr
        ------
        #{stderr}
      MSG

      super(msg)
    end
  end
end
