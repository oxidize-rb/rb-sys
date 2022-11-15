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

      srcprefix = "$(srcdir)/#{builder.ext_dir}".chomp("/")
      RbConfig.expand(srcdir = srcprefix.dup)

      full_cargo_command = cargo_command(srcdir, builder)

      make_install = +<<~MAKE
        #{conditional_assign("RB_SYS_BUILD_DIR", File.join(Dir.pwd, ".rb-sys"))}
        #{conditional_assign("CARGO", "cargo")}
        #{conditional_assign("CARGO_BUILD_TARGET", builder.target)}
        #{conditional_assign("SOEXT", builder.so_ext)}

        # Determine the prefix Cargo uses for the lib.
        #{if_neq_stmt("$(SOEXT)", "dll")}
        #{conditional_assign("SOEXT_PREFIX", "lib", indent: 1)}
        #{endif_stmt}

        #{conditional_assign("RB_SYS_CARGO_PROFILE", builder.profile)}
        #{conditional_assign("RB_SYS_CARGO_FEATURES", builder.features.join(","))}
        #{conditional_assign("RB_SYS_GLOBAL_RUSTFLAGS", GLOBAL_RUSTFLAGS.join(" "))}
        #{conditional_assign("RB_SYS_EXTRA_RUSTFLAGS", builder.extra_rustflags.join(" "))}

        # Set dirname for the profile, since the profiles do not directly map to target dir (i.e. dev -> debug)
        #{if_eq_stmt("$(RB_SYS_CARGO_PROFILE)", "dev")}
        #{conditional_assign("RB_SYS_CARGO_PROFILE_DIR", "debug", indent: 1)}
        #{else_stmt}
        #{conditional_assign("RB_SYS_CARGO_PROFILE_DIR", "$(RB_SYS_CARGO_PROFILE)", indent: 1)}
        #{endif_stmt}

        # Set the build profile (dev, release, etc.) Compat with Rust 1.51.
        #{if_eq_stmt("$(RB_SYS_CARGO_PROFILE)", "release")}
        #{assign_stmt("RB_SYS_CARGO_PROFILE_FLAG", "--release", indent: 1)}
        #{else_stmt}
        #{assign_stmt("RB_SYS_CARGO_PROFILE_FLAG", "--profile $(RB_SYS_CARGO_PROFILE)", indent: 1)}
        #{endif_stmt}

        # Account for sub-directories when using `--target` argument with Cargo
        #{if_neq_stmt("$(CARGO_BUILD_TARGET)", "")}
        #{assign_stmt("RB_SYS_CARGO_BUILD_TARGET_DIR", "target/$(CARGO_BUILD_TARGET)", indent: 1)}
        #{else_stmt}
        #{assign_stmt("RB_SYS_CARGO_BUILD_TARGET_DIR", "target", indent: 1)}
        #{endif_stmt}

        target_prefix = #{target_prefix}
        TARGET_NAME = #{target[/\A\w+/]}
        TARGET_ENTRY = #{RbConfig::CONFIG["EXPORT_PREFIX"]}Init_$(TARGET_NAME)
        RUBYARCHDIR   = $(sitearchdir)$(target_prefix)
        TARGET = #{target}
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}
        #{conditional_assign("TARGET_DIR", "$(RB_SYS_CARGO_BUILD_TARGET_DIR)")}
        RUSTLIB = $(TARGET_DIR)/$(RB_SYS_CARGO_PROFILE_DIR)/$(SOEXT_PREFIX)$(TARGET_NAME).$(SOEXT)

        CLEANOBJS = $(TARGET_DIR)/.fingerprint $(TARGET_DIR)/incremental $(TARGET_DIR)/examples $(TARGET_DIR)/deps $(TARGET_DIR)/build $(TARGET_DIR)/.cargo-lock $(TARGET_DIR)/*.d $(TARGET_DIR)/*.rlib $(RB_SYS_BUILD_DIR)
        DEFFILE = $(TARGET_DIR)/$(TARGET)-$(arch).def
        CLEANLIBS = $(DLLIB) $(RUSTLIB) $(DEFFILE)

        #{base_makefile(srcdir)}

        #{if_neq_stmt("$(RB_SYS_VERBOSE)", "")}
        #{assign_stmt("Q", "$(0=@)", indent: 1)}
        #{endif_stmt}

        #{env_vars(builder)}
        #{export_env("RUSTFLAGS", "$(RB_SYS_GLOBAL_RUSTFLAGS) $(RB_SYS_EXTRA_RUSTFLAGS) $(RUSTFLAGS)")}

        FORCE: ;

        $(TARGET_DIR):
        \t$(ECHO) creating target directory \\($(@)\\)
        \t$(Q) $(MAKEDIRS) $(TARGET_DIR)

        #{deffile_definition}

        #{optional_rust_toolchain(builder)}

        $(RUSTLIB): #{deffile_definition ? "$(DEFFILE) " : nil}FORCE
        \t$(ECHO) generating $(@) \\("$(RB_SYS_CARGO_PROFILE)"\\)
        \t$(Q) #{full_cargo_command}

        $(DLLIB): $(RUSTLIB)
        \t$(Q) $(COPY) "$(RUSTLIB)" $@

        install-so: $(DLLIB)
        \t$(ECHO) installing $(DLLIB) to $(RUBYARCHDIR)
        \t$(Q) $(MAKEDIRS) $(RUBYARCHDIR)
        \t$(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

        install: #{builder.clean_after_install ? "install-so realclean" : "install-so"}

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
      Shellwords.join(cargo_cmd).gsub("\\=", "=").gsub(/\Acargo/, "$(CARGO)").gsub(/-v=\d/, "")
    end

    def env_vars(builder)
      lines = builder.build_env.map { |k, v| env_line(k, v) }
      lines << env_line("CC", env_or_makefile_config("CC"))
      lines << env_line("CXX", env_or_makefile_config("CXX"))
      lines << env_line("AR", env_or_makefile_config("AR")) unless env_or_makefile_config("AR") == "libtool -static"
      lines.compact.join("\n")
    end

    def env_line(k, v)
      return unless v
      export_env(k, v.gsub("\n", '\n'))
    end

    def env_or_makefile_config(key)
      ENV[key] || RbConfig::MAKEFILE_CONFIG[key]
    end

    def gsub_cargo_command!(cargo_command, builder:)
      cargo_command.gsub!(/--profile \w+/, "$(RB_SYS_CARGO_PROFILE_FLAG)")
      cargo_command.gsub!(%r{--features \S+}, "--features $(RB_SYS_CARGO_FEATURES)")
      cargo_command.gsub!(%r{--target \S+}, "--target $(CARGO_BUILD_TARGET)")
      cargo_command.gsub!(/--target-dir (?:(?!--).)+/, "--target-dir $(TARGET_DIR) ")
      cargo_command
    end

    def deffile_definition
      warn("EXPORT_PREFIX is not defined, please require \"mkmf\" before requiring \"rb_sys/mkmf\"") unless defined?(EXPORT_PREFIX)

      return unless defined?(EXPORT_PREFIX) && EXPORT_PREFIX

      @deffile_definition ||= <<~MAKE
        $(DEFFILE): $(TARGET_DIR)
        \t$(ECHO) generating $(@)
        \t$(Q) ($(COPY) $(srcdir)/$(TARGET).def $@ 2> /dev/null) || (echo EXPORTS && echo $(TARGET_ENTRY)) > $@
      MAKE
    end

    def optional_rust_toolchain(builder)
      <<~MAKE
        #{conditional_assign("RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN", builder.force_install_rust_toolchain)}

        # Only run if the we are told to explicitly install the Rust toolchain
        #{if_neq_stmt("$(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)", "false")}
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

        $(CARGO):
        \t$(Q) $(MAKEDIRS) $(CARGO_HOME) $(RUSTUP_HOME)
        \tcurl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL "https://sh.rustup.rs" | sh -s -- --no-modify-path --profile $(RB_SYS_RUSTUP_PROFILE) --default-toolchain none -y
        \trustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE)
        \trustup default $(RB_SYS_DEFAULT_TOOLCHAIN)

        $(RUSTLIB): $(CARGO)
        #{endif_stmt}
      MAKE
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
  end
end
# rubocop:enable Style/GlobalVars

include RbSys::Mkmf # rubocop:disable Style/MixinUsage
