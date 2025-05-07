# The Ruby on Rust Book

This directory contains the source for "The Ruby on Rust Book" which is built using
[mdBook](https://rust-lang.github.io/mdBook/).

## SEO Setup

This book has been configured with SEO optimizations:

1. **Meta Tags**: The theme has been updated to include proper meta tags for SEO, including Open Graph and Twitter card
   support.
2. **Sitemap Generation**: A sitemap.xml file is generated after the book is built.
3. **Robots.txt**: A robots.txt file is included to help search engines navigate the site.
4. **Custom CSS/JS**: Custom CSS and JS files are included to improve readability and add structured data for search
   engines.

## Building the Book

To build the book with all SEO features:

```bash
# Run the Rake task from the project root
rake book:build_with_sitemap
```

This will:

1. Build the book using mdBook
2. Generate the sitemap.xml file
3. Copy the robots.txt file to the output directory

## Submitting to Search Engines

After deploying the site, you should submit the sitemap to search engines:

- Google: Submit through [Google Search Console](https://search.google.com/search-console)
- Bing: Submit through [Bing Webmaster Tools](https://www.bing.com/webmasters)

## Customizing SEO

To customize the SEO settings:

1. Edit `book.toml` to update metadata like title and description
2. Modify `theme/index.hbs` to update meta tags
3. Update `theme/css/custom.css` and `theme/js/custom.js` for styling and behavior
4. Adjust the sitemap generation settings in `rakelib/book.rake`
