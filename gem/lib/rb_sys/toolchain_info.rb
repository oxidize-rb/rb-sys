# frozen_string_literal: true

require_relative "toolchain_info/data"

module RbSys
  class ToolchainInfo
    attr_reader :platform, :gem_platform, :rust_target, :rake_compiler_dock_cc, :supported, :rake_compiler_dock_image

    class << self
      def all
        @all ||= DATA.keys.map { |k| new(k) }
      end

      def local
        @current ||= new(Gem::Platform.local)
      end
    end

    def initialize(plat)
      @gem_platform = plat.is_a?(Gem::Platform) ? plat : Gem::Platform.new(plat)
      @platform = "#{@gem_platform.cpu}-#{@gem_platform.os}"
      data = DATA.fetch(platform) { raise ArgumentError, "unknown ruby platform: #{platform}" }
      @rust_target = data["rust-target"]
      @rake_compiler_dock_cc = data["rake-compiler-dock"]["cc"]
      @supported = data["supported"]
      @rake_compiler_dock_image = "rbsys/#{platform}:#{RbSys::VERSION}"
    end

    def supported?
      @supported
    end

    def to_s
      @platform
    end

    def ==(other)
      @gem_platform == other.gem_platform
    end
  end
end
