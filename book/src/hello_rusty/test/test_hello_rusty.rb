# frozen_string_literal: true

require "test_helper"

class TestHelloRusty < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::HelloRusty::VERSION
  end

  def test_hello
    result = HelloRusty.hello("World")
    assert_equal "Hello from Rust, World!", result
  end
end
