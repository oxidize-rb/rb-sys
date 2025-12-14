namespace :release do
  desc "Update docs and such with the current toolchain info"
  task :toolchain_info do
    puts "Updating toolchain info in README and Cargo.toml files"
    readme_old_contents = File.read("readme.md")
    toolchains = JSON.parse(File.read("data/toolchains.json"))
    msrv = toolchains["policy"]["minimum-supported-rust-version"]

    readme_new_contents = readme_old_contents.gsub(/<!--\s*toolchains\s*([^>]+)\s*-->([^<]+)<!--\s*\/toolchains\s*-->/) do
      path = $1.strip
      parts = path.split(".").compact.reject(&:empty?)
      value = toolchains.dig(*parts) || raise("No value for path: #{parts}")

      "<!-- toolchains #{path} -->#{value}<!-- /toolchains -->"
    end

    File.write("readme.md", readme_new_contents)

    Dir["crates/*/Cargo.toml"].each do |path|
      old_content = File.read(path)
      new_content = old_content.gsub(/^rust-version = "[^"]+"$/, "rust-version = \"#{msrv}\"")
      File.write(path, new_content)
    end
  end

  desc "Prepare the release"
  task prepare: ["data:derive", "release:toolchain_info"]

  desc "Bump the gem version"
  task bump: "release:prepare" do
    require_relative "../gem/lib/rb_sys/version"
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
    crates = ["rb-sys-build", "rb-sys"]

    crates.each do |crate|
      sh "cargo publish -p #{crate}" do |ok, res|
        next if ok

        already_published = res.exitstatus == 101
        if already_published
          puts "Skipping already published crate: #{crate}"
        else
          exit res.exitstatus
        end
      end
    end

    Dir.chdir("gem") do
      sh "bundle exec rake release"
    end

    require_relative "../gem/lib/rb_sys/version"

    sh "gh", "release", "create", "v#{RbSys::VERSION}", "--generate-notes"

    sh "open", "https://www.rubydoc.info/gems/rb_sys/#{RbSys::VERSION}"
  end

  desc "Run a dry run of the release"
  task :dry_run do
    crates = ["rb-sys-build", "rb-sys", "rb-sys-test-helpers-macros", "rb-sys-test-helpers", "rb-sys-env"]

    crates.each do |crate|
      system "cargo publish -p #{crate} --dry-run --allow-dirty"
    end
  end

  desc "Publish rb-sys-test-helpers"
  task :publish_test_helpers do
    crates = ["rb-sys-test-helpers-macros", "rb-sys-test-helpers"]

    crates.each do |crate|
      sh "cargo publish -p #{crate}"
    end
  end
end
