// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded "><a href="getting-started.html"><strong aria-hidden="true">1.</strong> Prerequisites and Installation</a></li><li class="chapter-item expanded "><a href="quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start: Your First Extension</a></li><li class="chapter-item expanded "><a href="project-setup.html"><strong aria-hidden="true">3.</strong> Project Structure</a></li><li class="chapter-item expanded "><a href="hello-rusty-documentation.html"><strong aria-hidden="true">4.</strong> Anatomy of a Rusty Ruby Gem: hello_rusty</a></li><li class="chapter-item expanded "><a href="development-approaches.html"><strong aria-hidden="true">5.</strong> Development Approaches</a></li><li class="chapter-item expanded "><a href="working-with-ruby-objects.html"><strong aria-hidden="true">6.</strong> Working with Ruby Objects</a></li><li class="chapter-item expanded "><a href="classes-and-modules.html"><strong aria-hidden="true">7.</strong> Ruby Classes and Modules</a></li><li class="chapter-item expanded "><a href="error-handling.html"><strong aria-hidden="true">8.</strong> Error Handling</a></li><li class="chapter-item expanded "><a href="memory-management.html"><strong aria-hidden="true">9.</strong> Memory Management &amp; Safety</a></li><li class="chapter-item expanded "><a href="build-process.html"><strong aria-hidden="true">10.</strong> The Build Process</a></li><li class="chapter-item expanded "><a href="cross-platform.html"><strong aria-hidden="true">11.</strong> Cross-Platform Development</a></li><li class="chapter-item expanded "><a href="testing.html"><strong aria-hidden="true">12.</strong> Testing Extensions</a></li><li class="chapter-item expanded "><a href="debugging.html"><strong aria-hidden="true">13.</strong> Debugging &amp; Troubleshooting</a></li><li class="chapter-item expanded "><a href="troubleshooting.html"><strong aria-hidden="true">14.</strong> Troubleshooting Guide</a></li><li class="chapter-item expanded "><a href="api-reference/rb-sys-features.html"><strong aria-hidden="true">15.</strong> rb-sys Crate Features</a></li><li class="chapter-item expanded "><a href="api-reference/rb-sys-gem-config.html"><strong aria-hidden="true">16.</strong> rb_sys Gem Configuration</a></li><li class="chapter-item expanded "><a href="api-reference/test-helpers.html"><strong aria-hidden="true">17.</strong> Test Helpers</a></li><li class="chapter-item expanded "><a href="community-support.html"><strong aria-hidden="true">18.</strong> Getting Help</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="https://github.com/oxidize-rb/rb-sys/blob/main/CONTRIBUTING.html">Contributing to rb-sys</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
