# frozen_string_literal: true

require "rubygems/ext"
require "shellwords"
require_relative "cargo_builder"
require_relative "mkmf/config"

# Root module
module RbSys
  # Helper class for creating Rust Makefiles
  module Mkmf
    # Helper for building Rust extensions by creating a Ruby compatible makefile
    # for Rust. By using this class, your rust extension will be 100% compatible
    # with the rake-compiler gem, which allows for easy cross compilation.
    #
    # @example Basic
    #   require 'mkmf'
    # . require 'rb_sys/mkmf'
    #
    # . create_rust_makefile("my_extension") #=> Generate a Makefile in the current directory
    #
    # @example Configure a custom build profile
    #   require 'mkmf'
    # . require 'rb_sys/mkmf'
    #
    # . create_rust_makefile("my_extension") do |r|
    # .   # All of these are optional
    # .   r.env = { 'FOO' => 'bar' }
    # .   r.profile = ENV.fetch('RB_SYS_CARGO_PROFILE', :dev).to_sym
    # .   r.features = %w[some_cargo_feature]
    # . end
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

      # rubocop:disable Style/GlobalVars
      make_install = +<<~MAKE
        RB_SYS_BUILD_DIR ?= #{File.join(Dir.pwd, ".rb-sys")}
        CARGO ?= cargo
        CARGO_BUILD_TARGET ?= #{builder.target}
        SOEXT ?= #{builder.so_ext}

        # Determine the prefix Cargo uses for the lib.
        ifneq ($(SOEXT),dll)
          SOEXT_PREFIX ?= lib
        endif

        RB_SYS_CARGO_PROFILE ?= #{builder.profile}
        RB_SYS_CARGO_FEATURES ?= #{builder.features.join(",")}
        RB_SYS_EXTRA_RUSTFLAGS ?= #{builder.extra_rustflags.join(" ")}

        # Set dirname for the profile, since the profiles do not directly map to target dir (i.e. dev -> debug)
        ifeq ($(RB_SYS_CARGO_PROFILE),dev)
          RB_SYS_CARGO_PROFILE_DIR ?= debug
        else
          RB_SYS_CARGO_PROFILE_DIR ?= $(RB_SYS_CARGO_PROFILE)
        endif

        # Set the build profile (dev, release, etc.) Compat with Rust 1.51.
        ifeq ($(RB_SYS_CARGO_PROFILE),release)
          RB_SYS_CARGO_PROFILE_FLAG = --release
        else
          RB_SYS_CARGO_PROFILE_FLAG = --profile $(RB_SYS_CARGO_PROFILE)
        endif

        # Account for sub-directories when using `--target` argument with Cargo
        ifneq ($(CARGO_BUILD_TARGET),)
          RB_SYS_CARGO_BUILD_TARGET_DIR ?= target/$(CARGO_BUILD_TARGET)
        else
          RB_SYS_CARGO_BUILD_TARGET_DIR ?= target
        endif

        target_prefix = #{target_prefix}
        TARGET_NAME = #{target[/\A\w+/]}
        TARGET_ENTRY = #{RbConfig::CONFIG["EXPORT_PREFIX"]}Init_$(TARGET_NAME)
        CLEANLIBS = $(RUSTLIB) $(DLLIB) $(DEFFILE)
        RUBYARCHDIR   = $(sitearchdir)$(target_prefix)
        TARGET = #{target}
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}
        TARGET_DIR = #{Dir.pwd}/$(RB_SYS_CARGO_BUILD_TARGET_DIR)/$(RB_SYS_CARGO_PROFILE_DIR)
        RUSTLIB = $(TARGET_DIR)/$(SOEXT_PREFIX)$(TARGET_NAME).$(SOEXT)

        DISTCLEANDIRS = $(TARGET_DIR) $(RB_SYS_BUILD_DIR)
        DEFFILE = $(TARGET_DIR)/$(TARGET)-$(arch).def
        #{base_makefile(srcdir)}

        ifneq ($(RB_SYS_VERBOSE),)
          Q = $(0=@)
        endif

        #{env_vars(builder)}
        $(DLLIB): export RUSTFLAGS := $(RUSTFLAGS) $(RB_SYS_EXTRA_RUSTFLAGS)

        FORCE: ;

        $(TARGET_DIR):
        \t$(ECHO) creating target directory \\($(@)\\)
        \t$(Q) $(MAKEDIRS) $(TARGET_DIR)

        $(DEFFILE): $(TARGET_DIR)
        \t$(ECHO) generating $(@)
        \t$(Q) ($(COPY) $(srcdir)/$(TARGET).def $@ 2> /dev/null) || (echo EXPORTS && echo $(TARGET_ENTRY)) > $@

        #{optional_rust_toolchain(builder)}

        $(DLLIB): $(DEFFILE) FORCE
        \t$(ECHO) generating $(@) \\("$(RB_SYS_CARGO_PROFILE)"\\)
        \t$(Q) #{full_cargo_command}
        \t$(Q) $(COPY) "$(RUSTLIB)" $@

        install: $(DLLIB) Makefile
        \t$(ECHO) installing $(DLLIB)
        \t$(Q) $(MAKEDIRS) $(RUBYARCHDIR)
        \t$(Q) $(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

        all: #{$extout ? "install" : "$(DLLIB)"}
      MAKE

      gsub_cargo_command!(make_install, builder: builder)

      File.write("Makefile", make_install)
    end
    # rubocop:enable Style/GlobalVars

    private

    def base_makefile(cargo_dir)
      base_makefile = dummy_makefile(__dir__).join("\n")
      base_makefile.gsub!("all install static install-so install-rb", "all static install-so install-rb")
      base_makefile.gsub!("clean-so::", "clean-so:\n\t-$(Q)$(RM) $(DLLIB)\n")
      base_makefile.gsub!(/^srcdir = .*$/, "srcdir = #{cargo_dir}")
      base_makefile
    end

    def cargo_command(cargo_dir, builder)
      builder.ext_dir = cargo_dir
      dest_path = File.join(Dir.pwd, "target")
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
      %($(DLLIB): export #{k} = #{v.gsub("\n", '\n')})
    end

    def env_or_makefile_config(key)
      ENV[key] || RbConfig::MAKEFILE_CONFIG[key]
    end

    def gsub_cargo_command!(cargo_command, builder:)
      cargo_command.gsub!(/--profile \w+/, "$(RB_SYS_CARGO_PROFILE_FLAG)")
      cargo_command.gsub!(%r{--features \S+}, "--features $(RB_SYS_CARGO_FEATURES)")
      cargo_command.gsub!(%r{--target \S+}, "--target $(CARGO_BUILD_TARGET)")
      target_dir = "target/#{builder.target}".chomp("/")
      cargo_command.gsub!(%r{/#{target_dir}/[^/]+}, "/$(RB_SYS_CARGO_BUILD_TARGET_DIR)/$(RB_SYS_CARGO_PROFILE_DIR)")
      cargo_command
    end

    def optional_rust_toolchain(builder)
      <<~MAKE
        RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN ?= #{builder.force_install_rust_toolchain}

        # Only run if the we are told to explicitly install the Rust toolchain
        ifneq ($(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN),false)
        RB_SYS_RUSTUP_PROFILE ?= minimal

        # If the user passed true, we assume stable Rust. Otherwise, use what
        # was specified (i.e. RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN=beta)
        ifeq ($(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN),true)
          RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN = stable
        endif

        # If a $RUST_TARGET is specified (i.e. for rake-compiler-dock), append
        # that to the profile.
        ifeq ($(RUST_TARGET),)
          RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)
        else
          RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)-$(RUST_TARGET)
        endif

        export CARGO_HOME ?= $(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/cargo
        export RUSTUP_HOME ?= $(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/rustup
        export PATH := $(CARGO_HOME)/bin:$(RUSTUP_HOME)/bin:$(PATH)
        export RUSTUP_TOOLCHAIN := $(RB_SYS_DEFAULT_TOOLCHAIN)
        export CARGO := $(CARGO_HOME)/bin/cargo

        $(CARGO): 
        \t$(Q) $(MAKEDIRS) $(CARGO_HOME) $(RUSTUP_HOME)
        \tcurl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL "https://sh.rustup.rs" | sh -s -- --default-toolchain none -y
        \trustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE)
        \trustup default $(RB_SYS_DEFAULT_TOOLCHAIN)

        $(DLLIB): $(CARGO)
        endif
      MAKE
    end
  end
end

include RbSys::Mkmf # rubocop:disable Style/MixinUsage
