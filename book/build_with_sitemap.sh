#!/bin/bash
# Script to build the book and generate the sitemap

set -e

# Build the book
echo "Building the book..."
mdbook build

# Generate the sitemap
echo "Generating sitemap..."
python3 generate_sitemap.py

# Copy robots.txt to the output directory
echo "Copying robots.txt to the output directory..."
cp src/robots.txt html/

echo "Done! The book has been built and the sitemap has been generated."
echo "The sitemap is available at html/sitemap.xml"
