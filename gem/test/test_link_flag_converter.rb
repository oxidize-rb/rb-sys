require "test_helper"
require "rb_sys/cargo_builder"

class TestLinkFlagConverter < Minitest::Test
  def test_link_arg_detection
    flags = "-L/opt/ruby/lib  -Wl,--compress-debug-sections=zlib -Wl,-undefined,dynamic_lookup"
    args = Shellwords.split(flags)
    args = args.flat_map { |arg| RbSys::CargoBuilder::LinkFlagConverter.convert(arg) }

    assert_equal ["-L", "native=/opt/ruby/lib", "-C", "link_arg=-Wl,-undefined,dynamic_lookup"], args
  end
end
