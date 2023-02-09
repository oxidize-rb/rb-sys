# frozen_string_literal: true

module RbSys
  module Util
    class Logger
      attr_accessor :io, :level

      def initialize(io: $stderr, level: :info)
        @io = ENV["GITHUB_ACTIONS"] ? $stdout : io
        @level = level
      end

      def error(message, **opts)
        add(:error, message, **opts)
      end

      def warn(message, **opts)
        add(:warn, message, **opts)
      end

      def info(message, **opts)
        add(:info, message, **opts)
      end

      def notice(message, **opts)
        add(:notice, message, **opts)
      end

      def trace(message, **opts)
        return unless level == :trace

        add(:trace, message, **opts)
      end

      def fatal(message, **opts)
        error(message, **opts)
        abort
      end

      private

      LEVEL_STYLES = {
        warn: ["⚠️", "\e[1;33m"],
        error: ["❌", "\e[1;31m"],
        info: ["ℹ️", "\e[1;37m"],
        notice: ["🐳", "\e[1;34m"],
        trace: ["🔍", "\e[1;2m"]
      }

      if ENV["GITHUB_ACTIONS"]
        def add(level, message, emoji: true)
          emote, _ = LEVEL_STYLES.fetch(level.to_sym)
          io.puts "::#{level}::#{emote} #{message}"
        end
      else
        def add(level, message, emoji: true)
          emoji_opt, shellcode = LEVEL_STYLES.fetch(level.to_sym)

          emoji_opt = if emoji.is_a?(String)
            emoji + " "
          elsif emoji
            emoji_opt + " "
          end

          # Escape the message for bash shell codes (e.g. \033[1;31m)
          escaped = message.gsub("\\", "\\\\\\").gsub("\033", "\\033")

          io.puts "#{shellcode}#{emoji_opt}#{escaped}\033[0m"
        end
      end
    end
  end
end
