# Root module
module RbSys
  # Helper class for creating Rust Makefiles
  module Mkmf
    # Config that delegates to CargoBuilder if needded
    class Config
      attr_accessor :force_install_rust_toolchain

      def initialize(builder)
        @builder = builder
        @force_install_rust_toolchain = false
      end

      def method_missing(name, *args, &blk)
        @builder.send(name, *args, &blk)
      end

      def respond_to_missing?(name, include_private = false)
        @builder.respond_to?(name) || super
      end
    end
  end
end
