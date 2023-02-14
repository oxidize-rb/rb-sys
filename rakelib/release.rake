namespace :release do
  desc "Bump the gem version"
  task bump: ["readme", "data:derive"] do
    require_relative "./../gem/lib/rb_sys/version"
    old_version = RbSys::VERSION

    printf "What is the new version (current: #{old_version})?: "
    new_version = $stdin.gets.chomp

    sh "fastmod", "--extensions=md", "--accept-all", old_version.to_s, new_version.to_s
    sh "fastmod", "--extensions=toml", "--accept-all", "^version = \"#{old_version}\"", "version = #{new_version.inspect}"
    sh "fastmod", "--extensions=toml", "--accept-all", "^rb-sys = \\{ version = \"#{old_version}\"", "rb-sys = { version = #{new_version.inspect}"
    sh "fastmod", "--extensions=toml", "--accept-all", "^rb-sys-build = \\{ version = \"#{old_version}\"", "rb-sys-build = { version = #{new_version.inspect}"
    sh "fastmod", "--extensions=rb", "--accept-all", "^  VERSION = \"#{old_version}\"", "  VERSION = #{new_version.inspect}"
    sh "cargo check"
    Dir.chdir("examples/rust_reverse") { sh("cargo", "check") }
    sh "bundle"
    sh "rake test:examples"
  end

  desc "Publish the crates and gems"
  task :publish do
    crates = ["rb-sys-build", "rb-sys-local", "rb-sys"]

    crates.each do |crate|
      sh "cargo publish -p #{crate}"
    end

    Dir.chdir("gem") do
      sh "bundle exec rake release"
    end

    require_relative "./../gem/lib/rb_sys/version"

    sh "gh", "release", "create", "v#{RbSys::VERSION}", "--generate-notes"

    sh "open", "https://www.rubydoc.info/gems/rb_sys/#{RbSys::VERSION}"
  end
end
