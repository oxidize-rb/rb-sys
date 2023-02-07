require "test_helper"
require "rb_sys"
require "rb_sys/cargo/metadata"

class TestCargoMetadata < Minitest::Test
  def test_cargo_metadata_returns_valid_info
    skip if win_target?

    in_new_crate("foo") do |dir|
      metadata = RbSys::Cargo::Metadata.new("foo")

      assert_equal "foo", metadata.name
      assert metadata.target_directory.end_with?(File.join(dir, "target"))
      assert "0.1.0", metadata.version
    end
  end

  def test_fails_when_cargo_metadata_fails
    skip if win_target?

    Dir.mktmpdir do |dir|
      Dir.chdir(dir) do
        err = assert_raises(RbSys::CargoMetadataError) do
          metadata = RbSys::Cargo::Metadata.new("foo")
          metadata.target_directory
        end

        assert_match(/Could not infer Rust crate information using `cargo metadata`/, err.message)
      end
    end
  end

  private

  def in_new_crate(name, &blk)
    Dir.mktmpdir do |dir|
      Dir.chdir(dir) do
        `cargo new #{name} > /dev/null 2>&1`

        Dir.chdir(name) do
          yield File.join(dir, name)
        end
      end
    end
  end
end
