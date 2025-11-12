// Add small arrow icons to external links in the footer
export default function setupFooterIcons() {
  const footerLinks = document.querySelectorAll('.footer__link-item[target="_blank"]');

  footerLinks.forEach((link) => {
    // Don't add icon if it already has one
    if (!link.querySelector("svg")) {
      const icon = document.createElement("span");
      icon.innerHTML = `
        <svg width="12" height="12" aria-hidden="true" viewBox="0 0 24 24" style="margin-left: 4px; display: inline-block; vertical-align: middle; opacity: 0.7;">
          <path
            fill="currentColor"
            d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z"
          />
        </svg>
      `;
      link.appendChild(icon);
    }
  });
}
