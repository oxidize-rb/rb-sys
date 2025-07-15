namespace :docsite do
  desc "Build the documentation site with Docusaurus"
  task :build do
    Dir.chdir(File.expand_path("../docsite", __dir__)) do
      sh "npm install"
      sh "npm run build"
    end
  end

  desc "Run the docsite development server"
  task :serve do
    Dir.chdir(File.expand_path("../docsite", __dir__)) do
      sh "npm install"
      sh "npm run start"
    end
  end

  desc "Clean the built docsite"
  task :clean do
    Dir.chdir(File.expand_path("../docsite", __dir__)) do
      sh "npm run clear"
    end
  end

  namespace :migration do
    desc "Install dependencies needed for docsite"
    task :install do
      Dir.chdir(File.expand_path("../docsite", __dir__)) do
        sh "npm install"
      end
    end

    desc "Run the full migration from book to docsite (build both to ensure compatibility)"
    task run: [:install] do
      puts "Building the mdBook book (legacy)"
      Rake::Task["book:build"].invoke

      puts "Building the Docusaurus docsite (new)"
      Rake::Task["docsite:build"].invoke

      puts "Migration complete!"
      puts "   Legacy book built at: #{File.expand_path("../book/book", __dir__)}"
      puts "   New docsite built at: #{File.expand_path("../docsite/build", __dir__)}"
    end
  end
end

desc "Alias for docsite:serve"
task docsite: ["docsite:serve"]
