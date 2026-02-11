#!/usr/bin/env python3
"""Scrape eBay listings for good deals on surround sound speaker sets."""

import re
import sys
import time
from urllib.parse import quote_plus

try:
    import requests
    from bs4 import BeautifulSoup
except ImportError:
    print("Installing required packages...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests", "beautifulsoup4"])
    import requests
    from bs4 import BeautifulSoup


SEARCH_QUERIES = [
    "surround sound speaker system 5.1",
    "surround sound speaker set 7.1",
    "home theater speaker system",
    "surround sound speaker package high end",
]

QUALITY_BRANDS = [
    "bose", "sonos", "klipsch", "polk", "yamaha", "denon", "sony",
    "jbl", "harman kardon", "samsung", "lg", "onkyo", "pioneer",
    "definitive technology", "svs", "kef", "b&w", "bowers", "paradigm",
    "martin logan", "elac", "monoprice", "vizio", "enclave",
]

HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/120.0.0.0 Safari/537.36"
    ),
    "Accept-Language": "en-US,en;q=0.9",
}

MAX_PRICE = 500  # Upper price threshold for "deal" filtering
MIN_PRICE = 50   # Filter out suspiciously cheap listings


def build_url(query, page=1):
    """Build eBay search URL with filters for Buy It Now and price range."""
    encoded = quote_plus(query)
    return (
        f"https://www.ebay.com/sch/i.html?_nkw={encoded}"
        f"&_sacat=0&LH_BIN=1&_sop=15&_pgn={page}"
        f"&_udlo={MIN_PRICE}&_udhi={MAX_PRICE}&rt=nc&LH_ItemCondition=1000|1500|2000|2500|3000"
    )


def parse_price(text):
    """Extract numeric price from text like '$149.99' or '$80.00 to $120.00'."""
    prices = re.findall(r"\$[\d,]+\.?\d*", text)
    if not prices:
        return None
    # Use the first (lowest) price if a range is given
    return float(prices[0].replace("$", "").replace(",", ""))


def is_quality_brand(title):
    """Check if the listing title contains a known quality brand."""
    lower = title.lower()
    return any(brand in lower for brand in QUALITY_BRANDS)


def scrape_listings(query, max_pages=2):
    """Scrape eBay search results for a given query."""
    listings = []

    for page in range(1, max_pages + 1):
        url = build_url(query, page)
        print(f"  Fetching page {page}: {query}")

        try:
            resp = requests.get(url, headers=HEADERS, timeout=15)
            resp.raise_for_status()
        except requests.RequestException as e:
            print(f"  [Error] {e}")
            continue

        soup = BeautifulSoup(resp.text, "html.parser")
        items = soup.select("li.s-item")

        for item in items:
            title_el = item.select_one(".s-item__title")
            price_el = item.select_one(".s-item__price")
            link_el = item.select_one("a.s-item__link")
            shipping_el = item.select_one(".s-item__shipping, .s-item__freeXDays")
            condition_el = item.select_one(".SECONDARY_INFO")

            if not title_el or not price_el or not link_el:
                continue

            title = title_el.get_text(strip=True)
            if title.lower().startswith("shop on ebay"):
                continue

            price = parse_price(price_el.get_text())
            if price is None:
                continue

            link = link_el.get("href", "")
            shipping = shipping_el.get_text(strip=True) if shipping_el else "N/A"
            condition = condition_el.get_text(strip=True) if condition_el else "N/A"

            # Calculate a simple deal score
            brand_match = is_quality_brand(title)
            free_shipping = "free" in shipping.lower()
            deal_score = 0
            if brand_match:
                deal_score += 3
            if free_shipping:
                deal_score += 2
            if price < 200:
                deal_score += 2
            elif price < 350:
                deal_score += 1
            if "new" in condition.lower() or "open box" in condition.lower():
                deal_score += 1

            listings.append({
                "title": title,
                "price": price,
                "shipping": shipping,
                "condition": condition,
                "link": link,
                "brand_match": brand_match,
                "deal_score": deal_score,
            })

        time.sleep(1.5)  # Be polite between requests

    return listings


def deduplicate(listings):
    """Remove duplicate listings by link URL."""
    seen = set()
    unique = []
    for item in listings:
        key = item["link"].split("?")[0]
        if key not in seen:
            seen.add(key)
            unique.append(item)
    return unique


def main():
    print("=" * 70)
    print("  eBay Surround Sound Speaker Deal Finder")
    print("=" * 70)
    print(f"  Price range: ${MIN_PRICE} - ${MAX_PRICE} | Filter: Buy It Now\n")

    all_listings = []
    for query in SEARCH_QUERIES:
        results = scrape_listings(query, max_pages=2)
        all_listings.extend(results)

    all_listings = deduplicate(all_listings)

    # Sort by deal score (highest first), then by price (lowest first)
    all_listings.sort(key=lambda x: (-x["deal_score"], x["price"]))

    # Show top deals
    top = [l for l in all_listings if l["deal_score"] >= 3]
    if not top:
        top = all_listings[:15]

    print(f"\n{'=' * 70}")
    print(f"  TOP DEALS FOUND: {len(top)} (out of {len(all_listings)} total)")
    print(f"{'=' * 70}\n")

    for i, item in enumerate(top[:20], 1):
        brand_tag = " [QUALITY BRAND]" if item["brand_match"] else ""
        print(f"  #{i} (Score: {item['deal_score']}){brand_tag}")
        print(f"  Title:     {item['title'][:80]}")
        print(f"  Price:     ${item['price']:.2f}")
        print(f"  Shipping:  {item['shipping']}")
        print(f"  Condition: {item['condition']}")
        print(f"  Link:      {item['link'][:100]}")
        print()

    print(f"  Total listings scraped: {len(all_listings)}")
    print(f"  Showing top {min(len(top), 20)} deals")
    print("=" * 70)


if __name__ == "__main__":
    main()
