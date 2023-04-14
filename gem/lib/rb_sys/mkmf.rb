# frozen_string_literal: true

require "rubygems/ext"
require "shellwords"
require_relative "cargo_builder"
require_relative "mkmf/config"

# rubocop:disable Style/GlobalVars
# Root module
module RbSys
  # Helper class for creating Rust Makefiles
  module Mkmf
    # @api private
    GLOBAL_RUSTFLAGS = ["--cfg=rb_sys_gem"]

    # Helper for building Rust extensions by creating a Ruby compatible makefile
    # for Rust. By using this class, your rust extension will be 100% compatible
    # with the rake-compiler gem, which allows for easy cross compilation.
    #
    # @example Basic
    #   require 'mkmf'
    #   require 'rb_sys/mkmf'
    #
    #   create_rust_makefile("my_extension") #=> Generate a Makefile in the current directory
    #
    # @example Configure a custom build profile
    #   require 'mkmf'
    #   require 'rb_sys/mkmf'
    #
    #   create_rust_makefile("my_extension") do |r|
    #     # All of these are optional
    #     r.env = { 'FOO' => 'bar' }
    #     r.profile = ENV.fetch('RB_SYS_CARGO_PROFILE', :dev).to_sym
    #     r.features = %w[some_cargo_feature]
    #     r.rustflags = %w[--cfg=foo]
    #     r.target_dir = "some/target/dir"
    #   end
    def create_rust_makefile(target, &blk)
      if target.include?("/")
        target_prefix, target = File.split(target)
        target_prefix[0, 0] = "/"
      else
        target_prefix = ""
      end

      spec = Struct.new(:name, :metadata).new(target, {})
      cargo_builder = CargoBuilder.new(spec)
      builder = Config.new(cargo_builder)

      yield builder if blk

      srcprefix = File.join("$(srcdir)", builder.ext_dir.gsub(/\A\.\/?/, "")).chomp("/")
      RbConfig.expand(srcdir = srcprefix.dup)

      full_cargo_command = cargo_command(srcdir, builder)

      make_install = +<<~MAKE
        #{conditional_assign("RB_SYS_BUILD_DIR", File.join(Dir.pwd, ".rb-sys"))}
        #{conditional_assign("CARGO", "cargo")}
        #{conditional_assign("CARGO_BUILD_TARGET", builder.target)}
        #{conditional_assign("SOEXT", builder.so_ext)}
        #{try_load_bundled_libclang(builder)}

        # Determine the prefix Cargo uses for the lib.
        #{if_neq_stmt("$(SOEXT)", "dll")}
        #{conditional_assign("SOEXT_PREFIX", "lib", indent: 1)}
        #{endif_stmt}

        #{set_cargo_profile(builder)}
        #{conditional_assign("RB_SYS_CARGO_FEATURES", builder.features.join(","))}
        #{conditional_assign("RB_SYS_GLOBAL_RUSTFLAGS", GLOBAL_RUSTFLAGS.join(" "))}
        #{conditional_assign("RB_SYS_EXTRA_RUSTFLAGS", builder.extra_rustflags.join(" "))}
        #{conditional_assign("RB_SYS_EXTRA_CARGO_ARGS", builder.extra_cargo_args.join(" "))}
        #{conditional_assign("RB_SYS_CARGO_MANIFEST_DIR", builder.manifest_dir)}

        # Set dirname for the profile, since the profiles do not directly map to target dir (i.e. dev -> debug)
        #{if_eq_stmt("$(RB_SYS_CARGO_PROFILE)", "dev")}
        #{conditional_assign("RB_SYS_CARGO_PROFILE_DIR", "debug", indent: 1)}
        #{else_stmt}
        #{conditional_assign("RB_SYS_CARGO_PROFILE_DIR", "$(RB_SYS_CARGO_PROFILE)", indent: 1)}
        #{endif_stmt}

        # Set the build profile (dev, release, etc.) Compat with Rust 1.54.
        #{if_eq_stmt("$(RB_SYS_CARGO_PROFILE)", "release")}
        #{assign_stmt("RB_SYS_CARGO_PROFILE_FLAG", "--release", indent: 1)}
        #{else_stmt}
        #{assign_stmt("RB_SYS_CARGO_PROFILE_FLAG", "--profile $(RB_SYS_CARGO_PROFILE)", indent: 1)}
        #{endif_stmt}

        # Account for sub-directories when using `--target` argument with Cargo
        #{conditional_assign("RB_SYS_CARGO_TARGET_DIR", "target")}
        #{if_neq_stmt("$(CARGO_BUILD_TARGET)", "")}
        #{assign_stmt("RB_SYS_FULL_TARGET_DIR", "$(RB_SYS_CARGO_TARGET_DIR)/$(CARGO_BUILD_TARGET)", indent: 1)}
        #{else_stmt}
        #{assign_stmt("RB_SYS_FULL_TARGET_DIR", "$(RB_SYS_CARGO_TARGET_DIR)", indent: 1)}
        #{endif_stmt}

        target_prefix = #{target_prefix}
        TARGET_NAME = #{target[/\A\w+/]}
        TARGET_ENTRY = #{RbConfig::CONFIG["EXPORT_PREFIX"]}Init_$(TARGET_NAME)
        RUBYARCHDIR = $(sitearchdir)$(target_prefix)
        TARGET = #{target}
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}
        RUSTLIBDIR = $(RB_SYS_FULL_TARGET_DIR)/$(RB_SYS_CARGO_PROFILE_DIR)
        RUSTLIB = $(RUSTLIBDIR)/$(SOEXT_PREFIX)$(TARGET_NAME).$(SOEXT)
        TIMESTAMP_DIR = .

        CLEANOBJS = $(RUSTLIBDIR) $(RB_SYS_BUILD_DIR)
        CLEANLIBS = $(DLLIB) $(RUSTLIB)
        RUBYGEMS_CLEAN_DIRS = $(CLEANOBJS) $(CLEANFILES) #{builder.rubygems_clean_dirs.join(" ")}

        #{base_makefile(srcdir)}

        .PHONY: gemclean

        #{if_neq_stmt("$(RB_SYS_VERBOSE)", "")}
        #{assign_stmt("Q", "$(0=@)", indent: 1)}
        #{endif_stmt}

        #{env_vars(builder)}
        #{export_env("RUSTFLAGS", "$(RB_SYS_GLOBAL_RUSTFLAGS) $(RB_SYS_EXTRA_RUSTFLAGS) $(RUSTFLAGS)")}

        FORCE: ;

        #{optional_rust_toolchain(builder)}

        #{timestamp_file("sitearchdir")}:
        \t$(Q) $(MAKEDIRS) $(@D) $(RUBYARCHDIR)
        \t$(Q) $(TOUCH) $@

        $(RUSTLIB): FORCE
        \t$(ECHO) generating $(@) \\("$(RB_SYS_CARGO_PROFILE)"\\)
        \t#{full_cargo_command}

        $(DLLIB): $(RUSTLIB)
        \t$(Q) $(COPY) "$(RUSTLIB)" $@

        install-so: $(DLLIB) #{timestamp_file("sitearchdir")}
        \t$(ECHO) installing $(DLLIB) to $(RUBYARCHDIR)
        \t$(Q) $(MAKEDIRS) $(RUBYARCHDIR)
        \t$(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

        gemclean:
        \t$(ECHO) Cleaning gem artifacts
        \t-$(Q)$(RM_RF) $(RUBYGEMS_CLEAN_DIRS) 2> /dev/null || true

        install: #{builder.clean_after_install ? "install-so gemclean" : "install-so"}

        all: #{$extout ? "install" : "$(DLLIB)"}
      MAKE

      gsub_cargo_command!(make_install, builder: builder)

      File.write("Makefile", make_install)
    end

    private

    def base_makefile(cargo_dir)
      base_makefile = dummy_makefile(__dir__).join("\n")
      base_makefile.gsub!("all install static install-so install-rb", "all static install-rb")
      base_makefile.gsub!(/^srcdir = .*$/, "srcdir = #{cargo_dir}")
      base_makefile
    end

    def cargo_command(cargo_dir, builder)
      builder.ext_dir = cargo_dir
      dest_path = builder.target_dir || File.join(Dir.pwd, "target")
      args = ARGV.dup
      args.shift if args.first == "--"
      cargo_cmd = builder.cargo_command(dest_path, args)
      cmd = Shellwords.join(cargo_cmd)
      cmd.gsub!("\\=", "=")
      cmd.gsub!(/\Acargo rustc/, "$(CARGO) rustc $(RB_SYS_EXTRA_CARGO_ARGS) --manifest-path $(RB_SYS_CARGO_MANIFEST_DIR)/Cargo.toml")
      cmd.gsub!(/-v=\d/, "")
      cmd
    end

    def env_vars(builder)
      lines = builder.build_env.map { |k, v| env_line(k, v) }
      lines << env_line("CC", strip_cmd(env_or_makefile_config("CC")))
      lines << env_line("CXX", strip_cmd(env_or_makefile_config("CXX")))
      lines << env_line("AR", strip_cmd(env_or_makefile_config("AR"))) unless env_or_makefile_config("AR") == "libtool -static"
      lines.compact.join("\n")
    end

    def env_line(k, v)
      return unless v
      export_env(k, v.gsub("\n", '\n'))
    end

    def strip_cmd(cmd)
      cmd.gsub("-nologo", "").strip
    end

    def env_or_makefile_config(key)
      ENV[key] || RbConfig::MAKEFILE_CONFIG[key]
    end

    def gsub_cargo_command!(cargo_command, builder:)
      cargo_command.gsub!(/--profile \w+/, "$(RB_SYS_CARGO_PROFILE_FLAG)")
      cargo_command.gsub!(%r{--features \S+}, "--features $(RB_SYS_CARGO_FEATURES)")
      cargo_command.gsub!(%r{--target \S+}, "--target $(CARGO_BUILD_TARGET)")
      cargo_command.gsub!(/--target-dir (?:(?!--).)+/, "--target-dir $(RB_SYS_CARGO_TARGET_DIR) ")
      cargo_command
    end

    def rust_toolchain_env(builder)
      <<~MAKE
        #{conditional_assign("RB_SYS_RUSTUP_PROFILE", "minimal")}

        # If the user passed true, we assume stable Rust. Otherwise, use what
        # was specified (i.e. RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN=beta)
        #{if_eq_stmt("$(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)", "true")}
          RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN = stable
        #{endif_stmt}

        # If a $RUST_TARGET is specified (i.e. for rake-compiler-dock), append
        # that to the profile.
        #{if_eq_stmt("$(RUST_TARGET)", "")}
          RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)
        #{else_stmt}
          RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)-$(RUST_TARGET)
        #{endif_stmt}

        # Since we are forcing the installation of the Rust toolchain, we need
        # to set these env vars unconditionally for the build.
        #{export_env("CARGO_HOME", "$(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/cargo")}
        #{export_env("RUSTUP_HOME", "$(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/rustup")}
        #{export_env("PATH", "$(CARGO_HOME)/bin:$(RUSTUP_HOME)/bin:$(PATH)")}
        #{export_env("RUSTUP_TOOLCHAIN", "$(RB_SYS_DEFAULT_TOOLCHAIN)")}
        #{export_env("CARGO", "$(CARGO_HOME)/bin/cargo")}
      MAKE
    end

    def optional_rust_toolchain(builder)
      <<~MAKE
        #{conditional_assign("RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN", force_install_rust_toolchain?(builder))}

        # Only run if the we are told to explicitly install the Rust toolchain
        #{if_neq_stmt("$(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)", "false")}
        #{rust_toolchain_env(builder)}

        $(CARGO):
        \t$(Q) $(MAKEDIRS) $(CARGO_HOME) $(RUSTUP_HOME)
        \t$(Q) curl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL "https://sh.rustup.rs" | sh -s -- --no-modify-path --profile $(RB_SYS_RUSTUP_PROFILE) --default-toolchain none -y
        \t$(Q) $(CARGO_HOME)/bin/rustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE)
        \t$(Q) $(CARGO_HOME)/bin/rustup default $(RB_SYS_DEFAULT_TOOLCHAIN)

        $(RUSTLIB): $(CARGO)
        #{endif_stmt}
      MAKE
    end

    def force_install_rust_toolchain?(builder)
      return builder.force_install_rust_toolchain if builder.force_install_rust_toolchain
      return false unless builder.rubygems_invoked? && builder.auto_install_rust_toolchain

      find_executable("cargo").nil?
    end

    def if_eq_stmt(a, b)
      if $nmake
        "!IF #{a.inspect} == #{b.inspect}"
      else
        "ifeq (#{a},#{b})"
      end
    end

    def if_neq_stmt(a, b)
      if $nmake
        "!IF #{a.inspect} != #{b.inspect}"
      else
        "ifneq (#{a},#{b})"
      end
    end

    def else_stmt
      if $nmake
        "!ELSE"
      else
        "else"
      end
    end

    def endif_stmt
      if $nmake
        "!ENDIF"
      else
        "endif"
      end
    end

    def conditional_assign(a, b, export: false, indent: 0)
      if $nmake
        result = +"!IFNDEF #{a}\n#{a} = #{b}\n!ENDIF\n"
        result << export_env(a, b) if export
        result
      else
        "#{"\t" * indent}#{export ? "export " : ""}#{a} ?= #{b}"
      end
    end

    def assign_stmt(a, b, indent: 0)
      if $nmake
        "#{a} = #{b}"
      else
        "#{"\t" * indent}#{a} = #{b}"
      end
    end

    def export_env(k, v)
      if $nmake
        "!if [set #{k}=#{v}]\n!endif"
      else
        "export #{k} := #{v}"
      end
    end

    def try_load_bundled_libclang(_builder)
      require "libclang"
      assert_libclang_version_valid!
      export_env("LIBCLANG_PATH", Libclang.libdir)
    rescue LoadError
      # If we can't load the bundled libclang, just continue
    end

    def assert_libclang_version_valid!
      libclang_version = Gem::Version.new(Libclang.version)

      if libclang_version < Gem::Version.new("5.0.0")
        raise "libclang version 5.0.0 or greater is required (current #{libclang_version})"
      end

      if libclang_version >= Gem::Version.new("15.0.0")
        raise "libclang version > 14.0.0 or greater is required (current #{libclang_version})"
      end
    end

    def set_cargo_profile(builder)
      return assign_stmt("RB_SYS_CARGO_PROFILE", "release") if builder.rubygems_invoked?

      conditional_assign("RB_SYS_CARGO_PROFILE", builder.profile)
    end
  end
end
# rubocop:enable Style/GlobalVars

include RbSys::Mkmf # rubocop:disable Style/MixinUsage
