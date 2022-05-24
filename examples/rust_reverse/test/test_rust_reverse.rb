# frozen_string_literal: true

require "test_helper"

class TestRustReverse < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::RustReverse::VERSION
  end

  def test_it_does_something_useful
    assert_equal "dlrow olleh", RustReverse.reverse("hello world")
  end

  def test_stressing_it_out
    GC.stress = true

    expected = "a" * 10000

    1000.times do
      assert_equal expected, RustReverse.reverse("a" * 10000)
    end
  end
end
