# frozen_string_literal: true

# Root module
module RbSys
  # Helper class for creating Rust Makefiles
  module Mkmf
    # Config that delegates to CargoBuilder if needded
    class Config
      # Force the installation of the Rust toolchain when building
      attr_accessor :force_install_rust_toolchain

      # Clean artifacts after install (default: true if invoked by Rubygems)
      attr_accessor :clean_after_install

      # Target directory for cargo artifacts
      attr_accessor :target_dir

      # Automatically install the Rust toolchain when building (default: true)
      attr_accessor :auto_install_rust_toolchain

      # Directories to clean after installing with Rubygems
      attr_accessor :rubygems_clean_dirs

      def initialize(builder)
        @builder = builder
        @force_install_rust_toolchain = false
        @auto_install_rust_toolchain = true
        @clean_after_install = rubygems_invoked?
        @rubygems_clean_dirs = ["./cargo-vendor"]
      end

      # @api private
      def cross_compiling?
        RbConfig::CONFIG["CROSS_COMPILING"] == "yes"
      end

      # @api private
      def method_missing(name, *args, &blk)
        @builder.send(name, *args, &blk)
      end

      # @api private
      def respond_to_missing?(name, include_private = false)
        @builder.respond_to?(name) || super
      end

      # Seems to be the only way to reliably know if we were invoked by Rubygems.
      # We want to know this so we can cleanup the target directory after an
      # install, to remove bloat.
      # @api private
      def rubygems_invoked?
        ENV.key?("SOURCE_DATE_EPOCH")
      end
    end
  end
end
