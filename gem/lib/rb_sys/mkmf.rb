# frozen_string_literal: true

require "rubygems/ext"
require "rubygems/ext/cargo_builder"

# Root module
module RbSys
  # Helpers for creating a Ruby compatible makefile for Rust
  module Mkmf
    def create_rust_makefile(target, cargo_dir = Dir.pwd)
      # rubocop:disable Style/GlobalVars
      make_install = <<~MAKE
        target_prefix = /#{target}
        CARGO_PROFILE = release
        CLEANLIBS = target/ $(RUSTLIB) $(DLLIB)
        DLLIB = $(TARGET).#{RbConfig::CONFIG["DLEXT"]}
        RUBYARCHDIR   = $(sitearchdir)$(target_prefix)
        RUSTLIB = target/$(CARGO_PROFILE)/lib$(TARGET).#{RbConfig::CONFIG["SOEXT"]}
        TARGET = #{target}

        #{base_makefile(cargo_dir)}

        #{env_vars(cargo_dir, target)}

        FORCE: ;

        $(DLLIB): FORCE
        \t#{cargo_command(cargo_dir, target)}
        \tcp $(RUSTLIB) $@

        install: $(DLLIB)
        \t$(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

        all: #{$extout ? "install" : "$(DLLIB)"}
      MAKE

      File.write(File.join(cargo_dir, "Makefile"), make_install)
    end
    # rubocop:enable Style/GlobalVars

    private

    def base_makefile(cargo_dir)
      base_makefile = dummy_makefile(cargo_dir).join("\n")
      base_makefile.gsub!("all install static install-so install-rb", "all static install-so install-rb")
      base_makefile.gsub!("clean-so::", "clean-so:\n\t-$(Q)$(RM) $(DLLIB)\n")
      base_makefile
    end

    def cargo_command(cargo_dir, target)
      spec = Struct.new(:name, :metadata).new(target, {})
      builder = Gem::Ext::CargoBuilder.new(spec)
      dest_path = File.join(cargo_dir, "target")
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

Kernel.include RbSys::Mkmf
