# frozen_string_literal: true

require_relative "lib/rb_sys/version"

Gem::Specification.new do |spec|
  spec.name = "rb_sys"
  spec.version = RbSys::VERSION
  spec.authors = ["Ian Ker-Seymer"]
  spec.email = ["i.kerseymer@gmail.com"]

  spec.summary = "Helpers for compiling Rust extensions for ruby"
  spec.homepage = "https://oxidize-rb.github.io/rb-sys/"
  spec.licenses = ["MIT", "Apache-2.0"]
  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/oxidize-rb/rb-sys"

  spec.files = Dir.glob("{lib,exe,certs}/**/*") + ["LICENSE-MIT", "LICENSE-APACHE", ".yardopts", "README.md"]
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Security
  spec.cert_chain = ["certs/ianks.pem"]
  unless ENV["RUBYGEMS_FORCE_DISABLE_GEM_SIGNING"] == "true"
    spec.signing_key = File.expand_path("~/.ssh/gem-private_key.pem") if $0.end_with?("gem")
  end

  # Needed for `RakeCompilerDock.cross_rubies`
  spec.add_dependency("rake-compiler-dock", "1.11.1")

  spec.metadata["rubygems_mfa_required"] = "true"
end
