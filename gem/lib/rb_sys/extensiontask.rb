require_relative "cargo/metadata"
require_relative "error"

begin
  require "rake/extensiontask"
rescue LoadError
  abort "Please install rake-compiler to use this feature"
end

module RbSys
  # ExtensionTask is a Rake::ExtensionTask subclass that is used to tailored for
  # Rust extensions. It has the same options a `Rake::ExtensionTask`.
  #
  # @see https://www.rubydoc.info/gems/rake-compiler/Rake/ExtensionTask
  #
  # @example
  #   RbSys::ExtensionTask.new("my-crate", my_gemspec) do |ext|
  #     ext.lib_dir = "lib/my-crate"
  #   end
  #
  # @param name [String] the crate name to build
  # @param gem_spec [Gem::Specification] the gem specification to build (needed for cross-compiling)
  # @return [Rake::ExtensionTask]
  class ExtensionTask < Rake::ExtensionTask
    def init(name = nil, gem_spec = :undefined)
      super(name, lint_gem_spec(name, gem_spec))

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

      CLEAN.include(target_directory) if defined?(CLEAN)
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
        ENV["RB_SYS_CARGO_PROFILE"] = "dev"
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

    private

    def lint_gem_spec(name, gs)
      gem_spec = case gs
      when :undefined
        return
      when Gem::Specification
        gs
      when String
        Gem::Specification.load(gem_spec) || raise(ArgumentError, "Unable to load gemspec from file #{gs.inspect}")
      else
        raise ArgumentError, "gem_spec must be a Gem::Specification, got #{gs.class}"
      end

      gem_spec.files.each do |f|
        if /\.(dll|so|dylib|lib|bundle)$/.match?(f)
          warn "⚠️ gemspec includes native artifact (#{f}), please remove it."
        end
      end

      if (gem_crate_name = gem_spec.metadata["cargo_crate_name"])
        if name != gem_crate_name
          warn "⚠️ cargo_crate_name (#{gem_crate_name}) does not match extension task crate name (#{name})"
        end
      end

      gem_spec
    end
  end
end
