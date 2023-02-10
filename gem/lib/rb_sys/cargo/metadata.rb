# frozen_string_literal: true

require "open3"
require "psych"

module RbSys
  module Cargo
    # Extracts metadata from a Cargo project using `cargo metadata`.
    class Metadata
      attr_reader :name

      # Initializes a new Cargo::Metadata instance.
      #
      # @param name [String] the name of the Cargo project
      def initialize(name, deps: false)
        raise ArgumentError, "name must be a String" unless name.is_a?(String)

        @name = name
        @cargo_metadata = nil
        @package_metadata = nil
        @deps = deps
      end

      # Returns the path where the Cargo project's Cargo.toml is located.
      #
      # @return [String]
      def manifest_directory
        @manifest_directory ||= File.dirname(manifest_path)
      end

      # Returns the target directory for the Cargo project.
      #
      # @return [String]
      def target_directory
        cargo_metadata.fetch("target_directory")
      end

      # Returns the workspace root for the Cargo project.
      #
      # @return [String]
      def workspace_root
        cargo_metadata.fetch("workspace_root")
      end

      # Returns the workspace members for the Cargo project.
      #
      # @return [Array<Hash>]
      def packages
        cargo_metadata.fetch("packages")
      end

      # Returns the path to the package's Cargo.toml.
      #
      # @return [String]
      def manifest_path
        package_metadata.fetch("manifest_path")
      end

      # Returns the package's version.
      #
      # @return [String]
      def version
        package_metadata.fetch("version")
      end

      # Returns the package's id.
      #
      # @return [String]
      def id
        package_metadata.fetch("id")
      end

      # Returns the package's Rust edition.
      #
      # @return [String]
      def edition
        package_metadata.fetch("edition")
      end

      # Returns the package's features.
      #
      # @return [Array<String>]
      def features
        package_metadata.fetch("features")
      end

      # Returns the package's custom metadata.
      #
      # @return [Hash]
      def metadata
        package_metadata.fetch("metadata")
      end

      # Returns the rb-sys version, if any.
      def rb_sys_version
        pkg = packages.find { |p| p.fetch("name") == "rb-sys" }
        return unless pkg
        pkg["version"]
      end

      private

      def package_metadata
        return @package_metadata if @package_metadata

        found = cargo_metadata.fetch("packages").find { |p| p.fetch("name") == name }
        raise PackageNotFoundError, @name unless found
        @package_metadata = found
      end

      def cargo_metadata
        return @cargo_metadata if @cargo_metadata

        ::Gem.load_yaml
        cargo = ENV["CARGO"] || "cargo"
        args = ["metadata", "--format-version", "1"]
        args << "--no-deps" unless @deps
        out, stderr, status = Open3.capture3(cargo, *args)
        raise "exited with non-zero status (#{status})" unless status.success?
        data = Gem::SafeYAML.safe_load(out)
        raise "metadata must be a Hash" unless data.is_a?(Hash)
        @cargo_metadata = data
      rescue => err
        raise CargoMetadataError.new(err, stderr)
      end
    end
  end
end
