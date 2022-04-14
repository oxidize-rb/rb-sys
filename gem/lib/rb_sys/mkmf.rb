# frozen_string_literal: true

require "rubygems/ext"
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

      srcprefix ||= "$(srcdir)/#{srcprefix}".chomp("/")
      RbConfig.expand(srcdir = srcprefix.dup)
      # rubocop:disable Style/GlobalVars
      make_install = <<~MAKE
        target_prefix = #{target_prefix}
        CARGO_PROFILE = release
        CLEANLIBS = target/ $(RUSTLIB) $(DLLIB)
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}
        RUBYARCHDIR   = $(sitearchdir)$(target_prefix)
        RUSTLIB = target/$(CARGO_PROFILE)/lib$(TARGET).#{RbConfig::CONFIG["SOEXT"]}
        TARGET = #{target}

        #{base_makefile(srcdir)}

        #{env_vars(srcdir, target)}

        FORCE: ;

        $(DLLIB): FORCE
        \t#{cargo_command(srcdir, target)}
        \tcp $(RUSTLIB) $@

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

    def cargo_command(cargo_dir, target)
      spec = Struct.new(:name, :metadata).new(target, {})
      builder = Gem::Ext::CargoBuilder.new(spec)
      dest_path = File.join(Dir.pwd, "target")
      args = []
      cargo_cmd = builder.cargo_command(cargo_dir, dest_path, args)
      cargo_cmd.join(" ")
    end

    def env_vars(cargo_dir, target)
      spec = Struct.new(:name, :metadata).new(target, {})
      builder = Gem::Ext::CargoBuilder.new(spec)
      builder.build_env.map { |k, v| %($(DLLIB): export #{k} = #{v.gsub("\n", '\n')}) }.join("\n")
    end
  end
end
