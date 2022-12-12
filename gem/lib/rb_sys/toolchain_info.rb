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
      def all
        @all ||= DATA.keys.map { |k| new(k) }
      end

      def supported
        @supported ||= all.select(&:supported?)
      end

      def local
        @current ||= new(RbConfig::CONFIG["arch"])
      end
    end

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

    def supported?
      @supported
    end

    def to_s
      "#{gem_platform.cpu}-#{gem_platform.os}"
    end

    def ==(other)
      @gem_platform == other.gem_platform
    end
  end
end
