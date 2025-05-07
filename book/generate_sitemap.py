#!/usr/bin/env python3
"""
Generate a sitemap.xml file for the mdBook site.
This script should be run after the book is built.
"""

import os
import datetime
import xml.dom.minidom
import xml.etree.ElementTree as ET
from pathlib import Path

# Configuration
SITE_URL = "https://oxidize-rb.github.io/rb-sys"  # Base URL of your site
BOOK_DIR = "book"  # Directory where the book is built
OUTPUT_DIR = os.path.join(BOOK_DIR, "html")  # Directory where the HTML files are
SITEMAP_PATH = os.path.join(OUTPUT_DIR, "sitemap.xml")  # Path to the sitemap.xml file
CHANGE_FREQ = "weekly"  # How frequently the page is likely to change
PRIORITY = "0.8"  # Priority of this URL relative to other URLs on your site

def generate_sitemap():
    """Generate a sitemap.xml file for the mdBook site."""
    # Create the root element
    urlset = ET.Element("urlset", xmlns="http://www.sitemaps.org/schemas/sitemap/0.9")
    
    # Get the current date in the format required by sitemaps
    today = datetime.datetime.now().strftime("%Y-%m-%d")
    
    # Walk through the output directory
    for root, _, files in os.walk(OUTPUT_DIR):
        for file in files:
            if file.endswith(".html") and file != "404.html" and file != "print.html":
                # Get the relative path from the output directory
                rel_path = os.path.relpath(os.path.join(root, file), OUTPUT_DIR)
                
                # Convert Windows path separators to URL path separators
                rel_path = rel_path.replace("\\", "/")
                
                # Create the URL
                if rel_path == "index.html":
                    url = SITE_URL
                else:
                    url = f"{SITE_URL}/{rel_path}"
                
                # Create the URL element
                url_element = ET.SubElement(urlset, "url")
                loc = ET.SubElement(url_element, "loc")
                loc.text = url
                lastmod = ET.SubElement(url_element, "lastmod")
                lastmod.text = today
                changefreq = ET.SubElement(url_element, "changefreq")
                changefreq.text = CHANGE_FREQ
                priority = ET.SubElement(url_element, "priority")
                priority.text = PRIORITY
    
    # Create the XML tree
    tree = ET.ElementTree(urlset)
    
    # Pretty print the XML
    xmlstr = xml.dom.minidom.parseString(ET.tostring(urlset)).toprettyxml(indent="  ")
    
    # Write the XML to the sitemap.xml file
    with open(SITEMAP_PATH, "w", encoding="utf-8") as f:
        f.write(xmlstr)
    
    print(f"Sitemap generated at {SITEMAP_PATH}")

if __name__ == "__main__":
    # Make sure the output directory exists
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # Generate the sitemap
    generate_sitemap()
