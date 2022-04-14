# frozen_string_literal: true

require_relative "lib/rb_sys/version"

Gem::Specification.new do |spec|
  spec.name = "rb_sys"
  spec.version = RbSys::VERSION
  spec.authors = ["Ian Ker-Seymer"]
  spec.email = ["i.kerseymer@gmail.com"]

  spec.summary = "Helpers for compiling Rust extensions for ruby"
  spec.homepage = "https://github.com/oxidize-rb/rb-sys"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 2.4.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/oxidize-rb/rb-sys"
  # spec.metadata['changelog_uri'] = "TODO: Put your gem's CHANGELOG.md URL here."

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject do |f|
      (f == __FILE__) || f.match(%r{\A(?:(?:bin|test|spec|features)/|\.(?:git|travis|circleci)|appveyor)})
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # spec.add_dependency "rubygems", "~> 3.4"

  # Security
  spec.cert_chain = ["certs/ianks.pem"]
  spec.signing_key = File.expand_path("~/.ssh/gem-private_key.pem") if /gem\z/.match?($0) # rubocop:disable Performance/EndWith
  spec.metadata = {"rubygems_mfa_required" => "true"}
end
