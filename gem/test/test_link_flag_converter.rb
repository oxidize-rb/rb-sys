require "test_helper"
require "rb_sys/cargo_builder"

class TestLinkFlagConverter < Minitest::Test
  def test_link_arg_detection
    flags = "-L/opt/ruby/lib  -Wl,--compress-debug-sections=zlib -Wl,-undefined,dynamic_lookup"
    args = Shellwords.split(flags)
    args = args.flat_map { |arg| RbSys::CargoBuilder::LinkFlagConverter.convert(arg) }

    assert_equal ["-L", "native=/opt/ruby/lib", "-C", "link-arg=-Wl,-undefined,dynamic_lookup"], args
  end

  def test_not_converting_static_libs
    flags = "-lshlwapi -l:libssp.a"
    args = Shellwords.split(flags)
    args = args.flat_map { |arg| RbSys::CargoBuilder::LinkFlagConverter.convert(arg) }

    assert_equal ["-l", "shlwapi", "-C", "link-arg=-l:libssp.a"], args
  end
end
