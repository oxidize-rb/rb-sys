# frozen_string_literal: true

require "rubygems/ext"
require "shellwords"
require_relative "./../../vendor/rubygems/ext/cargo_builder"

# Root module
module RbSys
  # Helpers for creating a Ruby compatible makefile for Rust
  module Mkmf
    def create_rust_makefile(target, srcprefix = nil)
      if target.include?("/")
        target_prefix, target = File.split(target)
        target_prefix[0, 0] = "/"
      else
        target_prefix = ""
      end

      spec = Struct.new(:name, :metadata).new(target, {})
      builder = Gem::Ext::CargoBuilder.new(spec)
      srcprefix ||= "$(srcdir)/#{srcprefix}".chomp("/")
      RbConfig.expand(srcdir = srcprefix.dup)

      # rubocop:disable Style/GlobalVars
      make_install = <<~MAKE
        target_prefix = #{target_prefix}
        CARGO_PROFILE = release
        CLEANLIBS = $(RUSTLIB) $(DLLIB)
        DISTCLEANDIRS = target/
        RUBYARCHDIR   = $(sitearchdir)$(target_prefix)
        RUSTLIB = #{dllib_path(builder)}
        TARGET = #{target}
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}

        #{base_makefile(srcdir)}

        #{env_vars(builder)}

        FORCE: ;

        $(DLLIB): FORCE
        \t#{cargo_command(srcdir, builder)}
        \t$(COPY) "$(RUSTLIB)" $@

        install: $(DLLIB)
        \t$(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

        all: #{$extout ? "install" : "$(DLLIB)"}
      MAKE

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
      dest_path = File.join(Dir.pwd, "target")
      args = []
      cargo_cmd = builder.cargo_command(cargo_dir, dest_path, args)
      Shellwords.join(cargo_cmd).gsub("\\=", "=")
    end

    def env_vars(builder)
      lines = builder.build_env.map { |k, v| env_line(k, v) }
      lines << env_line("CC", env_or_makefile_config("CC"))
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

    def dllib_path(builder)
      builder.cargo_dylib_path(File.join(Dir.pwd, "target"))
    end
  end
end

include RbSys::Mkmf # rubocop:disable Style/MixinUsage
