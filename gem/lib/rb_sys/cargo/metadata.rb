# frozen_string_literal: true

require "open3"
require "psych"
require "rubygems/safe_yaml"

module RbSys
  module Cargo
    class Metadata
      attr_reader :name

      def initialize(name)
        raise ArgumentError, "name must be a String" unless name.is_a?(String)

        @name = name
      end

      def self.delegates_to_cargo_metadata(*methods)
        methods.each do |method|
          define_method(method) { cargo_metadata.fetch(method.to_s) }
        end
      end

      def self.delegates_to_package_metadata(*methods)
        methods.each do |method|
          define_method(method) { package_metadata.fetch(method.to_s) }
        end
      end

      delegates_to_cargo_metadata :target_directory, :workspace_root, :packages

      delegates_to_package_metadata :manifest_path, :version, :id, :edition, :targets, :features, :metadata

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

        cargo = ENV["CARGO"] || "cargo"
        args = ["metadata", "--no-deps", "--format-version", "1"]
        out, stderr, _status = Open3.capture3(cargo, *args)
        @cargo_metadata = Gem::SafeYAML.safe_load(out)
      rescue => err
        raise CargoMetadataError.new(err, stderr, manifest_path)
      end
    end
  end
end
