# frozen_string_literal: true

module RbSys
  module Cargo
    # Reads Cargo configuration files to extract build settings.
    module Config
      class << self
        # Returns the build target from Cargo configuration files.
        # Searches from start_dir up to root, then checks CARGO_HOME.
        #
        # @param start_dir [String] the directory to start searching from
        # @return [String, nil] the target triple, or nil if not configured
        def build_target(start_dir: Dir.pwd)
          find_in_hierarchy(start_dir) || find_in_home
        end

        private

        def find_in_hierarchy(dir)
          dir = File.expand_path(dir)
          while (parent = File.dirname(dir)) != dir
            target = read_build_target(File.join(dir, ".cargo"))
            return target if target
            dir = parent
          end
          nil
        end

        def find_in_home
          cargo_home = ENV["CARGO_HOME"] || File.join(Dir.home, ".cargo")
          read_build_target(cargo_home)
        rescue ArgumentError
          # Dir.home can raise if HOME is not set
          nil
        end

        def read_build_target(cargo_dir)
          # Version without `.toml` is for cargo < 1.39
          %w[config.toml config].each do |name|
            path = File.join(cargo_dir, name)
            next unless File.exist?(path)
            target = parse_build_target(File.read(path))
            return target if target
          rescue Errno::EACCES, Errno::ENOENT
            next
          end
          nil
        end

        def parse_build_target(content)
          # That is a hacky way to avoid pulling a proper toml parser as a dependency
          current_section = nil
          content.each_line do |line|
            line = line.sub(/#.*/, "").strip
            next if line.empty?

            if line =~ /^\["?([^\]"]+)"?\]/
              current_section = $1.strip
            elsif current_section.nil? && line =~ /^"?build"?\."?target"?\s*=\s*["']([^"']+)["']/
              return $1
            elsif current_section.nil? && line =~ /^"?build"?\s*=\s*\{[^}]*"?target"?\s*=\s*["']([^"']+)["']/
              return $1
            elsif current_section == "build" && line =~ /^"?target"?\s*=\s*["']([^"']+)["']/
              return $1
            end
          end
          nil
        end
      end
    end
  end
end
