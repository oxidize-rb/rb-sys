namespace :book do
  desc "Run the book server"
  task :serve do
    sh "mdbook serve --open ./book"
  end

  desc "Run the book doctests"
  task :test do
    sh "mdbook test ./book"
  end

  desc "Build the book"
  task :build do
    sh "mdbook build ./book --open"
  end
end
