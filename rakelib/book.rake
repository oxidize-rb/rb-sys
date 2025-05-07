namespace :book do
  desc "Build the book with sitemap (default)"
  task default: :build

  desc "Run the book server"
  task :serve do
    sh "mdbook serve --open ./book"
  end

  desc "Run the book doctests"
  task :test do
    sh "mdbook test ./book"
  end

  desc "Build the book and generate sitemap"
  task :build do
    require "date"
    require "fileutils"

    # Configuration
    site_url = "https://oxidize-rb.github.io/rb-sys"
    book_dir = File.expand_path("../book", __dir__)
    output_dir = File.join(book_dir, "book")

    puts "Using output directory: #{output_dir}"
    sh "mdbook build --dest-dir #{output_dir} #{book_dir}"
    sitemap_path = File.join(output_dir, "sitemap.xml")
    change_freq = "weekly"
    priority = "0.8"
    indent = "  "

    # Make sure the output directory exists
    FileUtils.mkdir_p(output_dir)

    # Get the current date in the format required by sitemaps
    today = Date.today.to_s

    # Start building the XML content
    xml_content = <<~XML
      <?xml version="1.0" encoding="UTF-8"?>
      <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    XML

    # Walk through the output directory
    Dir.glob(File.join(output_dir, "**", "*.html")).each do |file_path|
      # Skip certain files
      next if file_path.end_with?("404.html", "print.html")

      # Get the relative path from the output directory
      rel_path = file_path.sub("#{output_dir}/", "")

      # Create the URL
      url = begin
        if rel_path == "index.html"
          site_url
        else
          "#{site_url}/#{rel_path}"
        end
      end

      # Create the URL element with proper indentation
      xml_content << "#{indent}<url>\n"
      xml_content << "#{indent}#{indent}<loc>#{xml_escape(url)}</loc>\n"
      xml_content << "#{indent}#{indent}<lastmod>#{today}</lastmod>\n"
      xml_content << "#{indent}#{indent}<changefreq>#{change_freq}</changefreq>\n"
      xml_content << "#{indent}#{indent}<priority>#{priority}</priority>\n"
      xml_content << "#{indent}</url>\n"
    end

    # Close the root element
    xml_content << "</urlset>\n"

    # Write the XML to the sitemap.xml file
    File.write(sitemap_path, xml_content)

    puts "Sitemap generated at #{sitemap_path}"

    # Copy robots.txt to the output directory
    FileUtils.cp(File.join(book_dir, "src", "robots.txt"), output_dir)
    puts "Copied robots.txt to the output directory"
  end

  def xml_escape(text)
    text.to_s.gsub(/[&<>"]/, {
      "&" => "&amp;",
      "<" => "&lt;",
      ">" => "&gt;",
      '"' => "&quot;"
    })
  end
end
