require_relative "cargo/metadata"
require_relative "error"

begin
  require "rake/extensiontask"
rescue LoadError
  abort "Please install rake-compiler to use this gem"
end

module RbSys
  class ExtensionTask < Rake::ExtensionTask
    def init(name = nil, gem_spec = nil)
      super
      @orginal_ext_dir = @ext_dir
      @ext_dir = cargo_metadata.manifest_directory
      @source_pattern = nil
      @compiled_pattern = "*.{obj,so,bundle,dSYM}"
      @cross_compile = ENV.key?("RUBY_TARGET")
      @cross_platform = [ENV["RUBY_TARGET"]].compact
      @cross_compiling_blocks = []
      @cross_compiling_blocks << proc do |gemspec|
        warn "Removing unneeded dependencies from native gemspec"
        gemspec.dependencies.reject! { |d| d.name == "rb_sys" }
      end
      @cross_compiling_blocks << proc do |gemspec|
        warn "Removing source files from native gemspec"
        gemspec.files.reject! { |f| f.end_with?(".rs") }
        gemspec.files.reject! { |f| f.match?(/Cargo.(toml|lock)$/) }
        gemspec.files.reject! { |f| extconf.end_with?(f) }
      end
    end

    def define
      super
      define_env_tasks
    end

    def cargo_metadata
      @cargo_metadata ||= Cargo::Metadata.new(@name)
    end

    def extconf
      File.join(cargo_metadata.manifest_directory, "extconf.rb")
    end

    def binary(_platf)
      super.tr("-", "_")
    end

    # I'm not sure why this is necessary, can it be removed?
    def source_files
      list = FileList[
        "#{ext_dir}/**/*.{rs,rb,c,h,toml}",
        "**/Cargo.{toml,lock}",
        "**/.cargo/**/*",
        "#{ext_dir}/lib/**/*"
      ]
      list.include("#{ext_dir}/#{@source_pattern}") if @source_pattern
      list.exclude(File.join(target_directory, "**/*"))
      list
    end

    def cross_compiling(&block)
      @cross_compiling_blocks << block if block
    end

    def target_directory
      cargo_metadata.target_directory
    end

    def define_native_tasks(for_platform = nil, ruby_ver = RUBY_VERSION, callback = nil)
      cb = proc do |gemspec|
        callback&.call(gemspec)

        @cross_compiling_blocks.each do |block|
          block.call(gemspec)
        end
      end

      super(for_platform, ruby_ver, cb)
    end

    def define_env_tasks
      task "rb_sys:env:default" do
        ENV["RB_SYS_CARGO_TARGET_DIR"] ||= target_directory
        ENV["RB_SYS_CARGO_MANIFEST_DIR"] ||= cargo_metadata.manifest_directory
        ENV["RB_SYS_CARGO_PROFILE"] ||= "release"
      end

      desc "Use the debug profile for building native Rust extensions"
      task "rb_sys:env:dev" do
        ENV["RB_SYS_CARGO_PROFILE"] = "debug"
      end

      desc "Use the release profile for building native Rust extensions"
      task "rb_sys:env:release" do
        ENV["RB_SYS_CARGO_PROFILE"] = "release"
      end

      file extconf => "rb_sys:env:default"

      desc 'Compile the native Rust extension with the "dev" profile'
      task "compile:dev" => ["rb_sys:env:dev", "compile"]

      desc 'Compile the native Rust extension with the "release" profile'
      task "compile:release" => ["rb_sys:env:release", "compile"]
    end
  end
end
