# frozen_string_literal: true

require "shellwords"

module RbSys
  class CargoBuilder < Gem::Ext::Builder
    # Converts Ruby link flags into something cargo understands
    class LinkFlagConverter
      FILTERED_PATTERNS = [
        /compress-debug-sections/, # Not supported by all linkers, and not required for Rust
        /^\s*-s\s*$/
      ]

      def self.convert(args)
        Shellwords.split(args).flat_map { |arg| convert_arg(arg) }
      end

      def self.convert_arg(arg)
        return [] if FILTERED_PATTERNS.any? { |p| p.match?(arg) }

        case arg.chomp
        when /^-L\s*(.+)$/
          ["-L", "native=#{$1}"]
        when /^--library=(\w+\S+)$/, /^-l\s*(\w+\S+)$/
          ["-l", $1]
        when /^-l\s*:lib(\S+).(so|dylib|dll)$/
          ["-l", "dylib=#{$1}"]
        when /^-F\s*(.*)$/
          ["-l", "framework=#{$1}"]
        else
          ["-C", "link-arg=#{arg}"]
        end
      end
    end
  end
end
