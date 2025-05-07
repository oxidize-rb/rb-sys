// Custom JavaScript for SEO and improved user experience

document.addEventListener("DOMContentLoaded", function () {
  // Add 'noopener' and 'noreferrer' to external links for security
  const externalLinks = document.querySelectorAll('a[href^="http"]');
  externalLinks.forEach((link) => {
    if (!link.getAttribute("rel")) {
      link.setAttribute("rel", "noopener noreferrer");
    }
  });

  // Add title attributes to links without them for better accessibility
  const linksWithoutTitle = document.querySelectorAll("a:not([title])");
  linksWithoutTitle.forEach((link) => {
    if (link.textContent.trim()) {
      link.setAttribute("title", link.textContent.trim());
    }
  });

  // Add alt attributes to images without them for better accessibility
  const imagesWithoutAlt = document.querySelectorAll("img:not([alt])");
  imagesWithoutAlt.forEach((img) => {
    const imgSrc = img.getAttribute("src");
    if (imgSrc) {
      const altText = imgSrc
        .split("/")
        .pop()
        .split(".")[0]
        .replace(/[-_]/g, " ");
      img.setAttribute("alt", altText);
    }
  });

  // Add structured data for search engines
  const structuredData = {
    "@context": "https://schema.org",
    "@type": "TechArticle",
    headline: document.title,
    description:
      document
        .querySelector('meta[name="description"]')
        ?.getAttribute("content") || "",
    author: {
      "@type": "Person",
      name:
        document
          .querySelector('meta[name="author"]')
          ?.getAttribute("content") || "Ian Ker-Seymer",
    },
    publisher: {
      "@type": "Organization",
      name: "rb-sys",
      logo: {
        "@type": "ImageObject",
        url: "https://oxidize-rb.github.io/rb-sys/favicon.png",
      },
    },
    url: window.location.href,
    mainEntityOfPage: {
      "@type": "WebPage",
      "@id": window.location.href,
    },
    datePublished: new Date().toISOString().split("T")[0],
    dateModified: new Date().toISOString().split("T")[0],
  };

  const script = document.createElement("script");
  script.type = "application/ld+json";
  script.textContent = JSON.stringify(structuredData);
  document.head.appendChild(script);
});
