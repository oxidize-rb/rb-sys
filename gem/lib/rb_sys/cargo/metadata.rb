# frozen_string_literal: true

require "open3"
require "psych"

module RbSys
  module Cargo
    # Extracts metadata from a Cargo project using `cargo metadata`.
    class Metadata
      attr_reader :name

      def initialize(name)
        raise ArgumentError, "name must be a String" unless name.is_a?(String)

        @name = name
      end

      # @api private
      def self.delegates_to_cargo_metadata(*methods)
        methods.each do |method|
          define_method(method) { cargo_metadata.fetch(method.to_s) }
        end
      end

      # @api private
      def self.delegates_to_package_metadata(*methods)
        methods.each do |method|
          define_method(method) { package_metadata.fetch(method.to_s) }
        end
      end

      delegates_to_cargo_metadata :target_directory, :workspace_root, :packages

      delegates_to_package_metadata :manifest_path, :version, :id, :edition, :targets, :features, :metadata

      # Returns the path where the Cargo project's Cargo.toml is located.
      #
      # @return [String]
      def manifest_directory
        @manifest_directory ||= File.dirname(manifest_path)
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
        args = ["metadata", "--no-deps", "--format-version", "1"]
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
