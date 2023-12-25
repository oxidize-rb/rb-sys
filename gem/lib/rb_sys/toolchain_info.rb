# frozen_string_literal: true

require_relative "toolchain_info/data"

module RbSys
  # A class to get information about the Rust toolchains, and how they map to
  # Ruby platforms.
  #
  # @example
  #   RbSys::ToolchainInfo.new("x86_64-unknown-linux-gnu").ruby_platform # => "x86_64-linux"
  #   RbSys::ToolchainInfo.new("x86_64-unknown-linux-gnu").supported? # => true
  #   RbSys::ToolchainInfo.new("x86_64-unknown-linux-gnu")
  class ToolchainInfo
    attr_reader :platform, :gem_platform, :rust_target, :rake_compiler_dock_cc, :supported, :rake_compiler_dock_image, :docker_platform

    class << self
      # Get all known toolchains.
      #
      # @return [Array<RbSys::ToolchainInfo>]
      def all
        @all ||= DATA.keys.map { |k| new(k) }
      end

      # Get all supported toolchains.
      #
      # @return [Array<RbSys::ToolchainInfo>]
      def supported
        @supported ||= all.select(&:supported?)
      end

      # Get the toolchain for the current platform.
      #
      # @return [RbSys::ToolchainInfo]
      def local
        @current ||= new(RUBY_PLATFORM.include?("java") ? RbConfig::CONFIG.values_at("target_cpu", "target_os").join("-") : RbConfig::CONFIG["arch"])
      end
    end

    # Create a new toolchain info object.
    #
    # @param platform [String] The platform to get information about.
    def initialize(platform)
      @platform = platform
      @gem_platform = Gem::Platform.new(platform)
      data = DATA[platform] || DATA["#{gem_platform.cpu}-#{gem_platform.os}"] || raise(ArgumentError, "unknown ruby platform: #{platform.inspect}")
      @rust_target = data["rust-target"]
      @rake_compiler_dock_cc = data["rake-compiler-dock"]["cc"]
      @supported = data["supported"]
      @rake_compiler_dock_image = "rbsys/#{platform}:#{RbSys::VERSION}"
      @docker_platform = data["docker-platform"]
    end

    # Whether this toolchain is supported.
    #
    # @return [Boolean]
    def supported?
      @supported
    end

    # String representation of the toolchain.
    #
    # @return [String]
    def to_s
      "#{gem_platform.cpu}-#{gem_platform.os}"
    end

    # Compare two toolchains.
    #
    # @param other [RbSys::ToolchainInfo]
    # @return [Boolean]
    def ==(other)
      @gem_platform == other.gem_platform
    end
  end
end
