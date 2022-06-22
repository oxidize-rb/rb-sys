# We need to require mkmr this since `rake-compiler` injects code here for cross compilation
require "mkmf"

# In a real gem, this would just be: `require "rb_sys/mkmf"`
require_relative "./../../../../gem/lib/rb_sys/mkmf"

create_rust_makefile("rust_reverse") do |r|
  # Create debug builds in dev. Make sure that release gems are compiled with
  # `RB_SYS_CARGO_PROFILE=release` (optional)
  r.profile = ENV.fetch("RB_SYS_CARGO_PROFILE", :dev).to_sym

  # Can be overridden with `RB_SYS_CARGO_FEATURES` env var (optional)
  r.features = ["test-feature"]

  # You can add whatever env vars you want to the env hash (optional)
  r.env = {"FOO" => "BAR"}
end
