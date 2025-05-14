# oxidize.rb Theme Customizations

This directory contains custom theme components for the oxidize.rb documentation site. These components override the default Docusaurus theme components to implement specific design requirements.

## Components

### Logo

- Replaces the broken `<img>` with an inline SVG logo
- Removes duplicate "oxidize.rb" text

### Navbar

- Hooks the hamburger icon's onClick to open/close the sidebar
- Simplifies the navbar to show only the logo and site title

### Tabs & Code Blocks

- Implements a real tab component that shows only one code block at a time
- Removes static headings and tab-wrapper icons

### Buttons

- Ensures both "Get Started" and "Installation Guide" use the same active styling
- Removes any `disabled` prop from the second button

### Layout

- Makes the sidebar mount and scroll independently
- Sets the main content container to `max-width: 72ch; margin: auto`

## Usage

These components are automatically used by Docusaurus through its swizzling mechanism. No additional configuration is needed.

## Implementation Details

- `Logo/index.tsx`: Custom logo component with inline SVG
- `Navbar/index.tsx`: Custom navbar with mobile sidebar toggle
- `Tabs/index.tsx`: Real tab component for code blocks
- `DocItem/Layout/index.tsx`: Custom layout for documentation pages
- `DocSidebar/Desktop/index.tsx`: Custom sidebar with independent scrolling
