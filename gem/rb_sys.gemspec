# frozen_string_literal: true

require_relative "lib/rb_sys/version"

Gem::Specification.new do |spec|
  spec.name = "rb_sys"
  spec.version = RbSys::VERSION
  spec.authors = ["Ian Ker-Seymer"]
  spec.email = ["i.kerseymer@gmail.com"]

  spec.summary = "Helpers for compiling Rust extensions for ruby"
  spec.homepage = "https://github.com/oxidize-rb/rb-sys"
  spec.licenses = ["MIT", "Apache-2.0"]
  spec.required_ruby_version = ">= 2.3.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/oxidize-rb/rb-sys"
  # spec.metadata['changelog_uri'] = "TODO: Put your gem's CHANGELOG.md URL here."

  # Specify which files should be added to the gem when it is released.
  spec.files = Dir.glob("{lib,certs,sig,vendor}/**/*") + ["LICENSE-MIT", "LICENSE-APACHE"]
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Security
  unless ENV["SKIP_GEM_SIGNING_FOR_LOCAL_INSTALL_ONLY"] == "true"
    spec.cert_chain = ["certs/ianks.pem"]
    spec.signing_key = File.expand_path("~/.ssh/gem-private_key.pem") if $0.end_with?("gem")
    spec.metadata = {"rubygems_mfa_required" => "true"}
  end
end
