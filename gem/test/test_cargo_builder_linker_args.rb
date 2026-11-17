require "test_helper"
require "rb_sys/cargo_builder"

class TestCargoBuilderLinkerArgs < Minitest::Test
  def test_uses_cc_as_linker
    builder = builder_with_config("CC" => "gcc -fPIC")

    assert_equal ["-C", "linker=gcc", "-C", "link-arg=-fPIC"], builder.send(:linker_args)
  end

  def test_uses_default_msvc_linker_for_cl
    builder = builder_with_config("CC" => "cl -nologo", "LIBRUBYARG_SHARED" => "x64-vcruntime140-ruby410.lib")

    assert_equal ["-l", "x64-vcruntime140-ruby410"], builder.send(:linker_args)
  end

  def test_uses_default_msvc_linker_for_clang_cl
    builder = builder_with_config("CC" => "clang-cl -nologo -source-charset:utf-8", "LIBRUBYARG_SHARED" => "x64-vcruntime140-ruby410.lib")

    assert_equal ["-l", "x64-vcruntime140-ruby410"], builder.send(:linker_args)
  end

  private

  def builder_with_config(config)
    builder = RbSys::CargoBuilder.new(Gem::Specification.new)
    builder.define_singleton_method(:makefile_config) { |name| config[name] }
    builder.define_singleton_method(:split_flags) { |_name| [] }
    builder
  end
end
