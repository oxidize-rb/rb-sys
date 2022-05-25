# We need to require mkmr this since `rake-compiler` injects code here for cross compilation
require "mkmf"

# In a real gem, this would just be: `require "rb_sys/mkmf"`
require_relative "./../../../../gem/lib/rb_sys/mkmf"

create_rust_makefile("rust_reverse/rust_reverse") do |r|
  # Create debug builds in dev. Make sure that release gems are compiled with `CARGO_BUILD_PROFILE=release`
  r.profile = ENV.fetch("CARGO_BUILD_PROFILE", :dev).to_sym
end
