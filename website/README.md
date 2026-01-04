# Voiceboard Website

Landing page for Voiceboard application.

## Structure

```
website/
├── index.html      # Main page
├── assets/
│   ├── logo.svg    # Logo (also used as favicon)
│   └── screenshot.png  # App screenshot (to be added)
└── README.md
```

## Development

Open `index.html` in a browser. No build step required.

For live reload during development:
```bash
# Using Python
python3 -m http.server 8000 --directory website

# Using Node.js (npx)
npx serve website
```

## Adding Screenshot

Replace `assets/screenshot.png` with an actual screenshot of the application.
Recommended size: 1200x800px or similar aspect ratio.

## Deployment

This is a static site. Deploy to any static hosting:

- **Vercel**: `vercel website/`
- **Netlify**: Drag & drop `website/` folder
- **Cloudflare Pages**: Connect repo, set `website` as root

## Features

- OS detection (Windows/macOS/Linux)
- Auto-fetch download links from GitHub Releases API
- Responsive design (mobile-friendly)
- Dark theme with Tailwind CSS
