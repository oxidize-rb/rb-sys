namespace :readme do
  task :toolchains do
    contents = File.read("readme.md")
    toolchains = JSON.parse(File.read("data/toolchains.json"))

    new_contents = contents.gsub(/<!--\s*toolchains (\S+)\s*-->[^<]*<!--\s*\/toolchains\s*-->/) do
      path = $1
      parts = path.split(".").compact.reject(&:empty?)
      value = toolchains.dig(*parts) || raise("No value for path: #{parts}")

      "<!--toolchains #{path} -->#{value}<!--/toolchains-->"
    end

    File.write("readme.md", new_contents)
  end
end

desc "Compile the README"
task readme: ["readme:toolchains"]
