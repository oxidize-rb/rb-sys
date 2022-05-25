require "mkmf"
require_relative "./../../../../gem/lib/rb_sys/mkmf"

create_rust_makefile("rust_reverse/rust_reverse") do |r|
  r.profile = ENV.fetch("CARGO_BUILD_PROFILE", :dev).to_sym
  r.env = {"NO_LINK_RUTIE" => "true"}
end
