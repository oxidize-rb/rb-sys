# frozen_string_literal: true

# rubocop:disable all

# add rename_task method to Rake::Application
# it has an internal hash with name -> Rake::Task mapping
module Rake
  class Application
    def rename_task(task, oldname, newname)
      @tasks = {} if @tasks.nil?
      @tasks[newname.to_s] = task

      @tasks.delete(oldname) if @tasks.has_key?(oldname)
    end
  end
end

# add new rename method to Rake::Task class
# to rename a task
class Rake::Task
  def rename(new_name)
    unless new_name.nil?
      old_name = @name

      return if old_name == new_name

      @name = new_name.to_s
      application.rename_task(self, old_name, new_name)
    end
  end
end

# rubocop:enable all
