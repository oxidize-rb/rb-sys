# Root module
module RbSys
  # Helper class for creating Rust Makefiles
  module Mkmf
    # Config that delegates to CargoBuilder if needded
    class Config
      attr_accessor :force_install_rust_toolchain, :clean_after_install, :target_dir

      def initialize(builder)
        @builder = builder
        @force_install_rust_toolchain = false
        @clean_after_install = rubygems_invoked?
      end

      def method_missing(name, *args, &blk)
        @builder.send(name, *args, &blk)
      end

      def respond_to_missing?(name, include_private = false)
        @builder.respond_to?(name) || super
      end

      private

      # Seems to be the only way to reliably know if we were invoked by Rubygems.
      # We want to know this so we can cleanup the target directory after an
      # install, to remove bloat.
      def rubygems_invoked?
        ENV.key?("SOURCE_DATE_EPOCH")
      end
    end
  end
end
