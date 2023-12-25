# frozen_string_literal: true

require "jruby"

# Name with underscore doesn't work for some reason, says RustReverseService not found
def rbsys
  Java::Rbsys
end

# like require_relative
lib_path = File.join(
  File.absolute_path(File.dirname(__FILE__)),
  "rust_reverse.#{RbConfig::MAKEFILE_CONFIG["DLEXT"]}"
)
rbsys.rust_reverse.RustReverseService.systemLoad(lib_path)
rbsys.rust_reverse.RustReverseService.new.basicLoad(JRuby.runtime)
