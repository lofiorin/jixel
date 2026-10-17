# Shipping JPEG XL on websites

This guide takes you from one image to a fast page on any stack: plain HTML,
Next.js / React, WordPress, and nginx or Apache servers.

- [How it works](#how-it-works)
- [1. Export](#1-export)
- [2. Plain HTML / any static site](#2-plain-html--any-static-site)
- [3. Next.js and React](#3-nextjs-and-react)
- [4. WordPress](#4-wordpress)
- [5. Server configuration](#5-server-configuration)
- [6. CDNs and caching](#6-cdns-and-caching)
- [7. Build pipelines and many images](#7-build-pipelines-and-many-images)
- [Checklist](#checklist)

## How it works

Browsers that support JPEG XL download the `.jxl` file; others pick the first
fallback they understand. This is done with the standard HTML `<picture>`
element, which every browser supports, so **no image is ever broken**:

```html
<picture>
  <source type="image/jxl"  srcset="/images/hero-480.jxl 480w, /images/hero-1080.jxl 1080w, …" sizes="100vw">
  <source type="image/avif" srcset="/images/hero-480.avif 480w, …" sizes="100vw">
  <img src="/images/hero-1080.jpg" srcset="/images/hero-480.jpg 480w, …" sizes="100vw"
       width="3840" height="2160" alt="…" loading="lazy" decoding="async">
</picture>
```

`srcset` + `sizes` let the browser choose the right width for the screen, so
phones don't download desktop images. `width` and `height` prevent layout
shift (CLS). `loading="lazy"` defers images below the fold; hero images use
`loading="eager" fetchpriority="high"` instead.

## 1. Export

In the app: **Optimize for web**, or with the CLI:

```bash
jixel web photo.jpg -o export/ --name hero --alt "Sunset over the bay" --hero
```

The export folder contains:

```
hero-480.jxl  hero-768.jxl  …  hero-2560.jxl
hero-480.avif …                hero-2560.avif
hero-480.jpg  …                hero-2560.jpg
manifest.json
snippets/picture.html
snippets/JxlPicture.tsx
snippets/wordpress.php
snippets/nginx.conf
snippets/apache.htaccess
```

**Choosing options**

- *Widths*: the defaults (480, 768, 1080, 1440, 1920, 2560) cover phones to
  large desktop screens at 1× and 2×. For an image shown at most 600 px wide,
  480/768/1080/1440 is enough.
- *Sizes*: tell the browser how wide the image is displayed. Full-width:
  `100vw`. Half on desktop: `(min-width: 1024px) 50vw, 100vw`.
- *Quality*: 85 is a good default for photos (SSIMULACRA2 ≈ 80–85).
- *Fallbacks*: AVIF + JPEG (default) gives the smallest files everywhere. WebP
  + JPEG if you need to avoid AVIF encoding time. JPEG only is the simplest.

## 2. Plain HTML / any static site

1. Upload the folder (without `snippets/` if you like) to the URL path you set
   as *Public URL path*, e.g. `/images/`.
2. Paste `snippets/picture.html` where the image goes.
3. Make sure your server sends the right MIME type for `.jxl` (see
   [server configuration](#5-server-configuration)). Most hosts already do;
   if JPEG XL images don't show in Safari, this is the reason.

Works with Astro, Hugo, Eleventy, Jekyll, SvelteKit, Nuxt, Laravel Blade,
Django templates, Rails views… anything that outputs HTML.

## 3. Next.js and React

`next/image` re-encodes images through its optimiser, which doesn't produce
JPEG XL, so Jixel generates a small component that renders `<picture>`
directly. It works in the App Router and Pages Router (server component, no
`"use client"`), and in Remix, Gatsby, Vite, CRA.

1. Copy the export to `public/images/hero/` (keep `manifest.json`).
2. Copy `snippets/JxlPicture.tsx` to `components/JxlPicture.tsx` (once per
   project).
3. Use it:

```tsx
import JxlPicture from "@/components/JxlPicture";
import hero from "@/public/images/hero/manifest.json";

export default function Page() {
  return <JxlPicture manifest={hero} alt="Sunset over the bay" priority sizes="100vw" />;
}
```

Set the export's *Public URL path* to the URL the folder is served from
(`/images/hero/` in this example).

Props: `manifest`, `alt`, `sizes`, `priority` (eager + high priority),
`className`, `style`.

If you deploy on Vercel or Netlify, `.jxl` files in `public/` are served with
the right MIME type automatically.

## 4. WordPress

WordPress core rejects `.jxl` uploads. The generated `wordpress.php` is a
*must-use plugin* that:

- allows `.jxl` uploads in the Media Library,
- adds a `[jixel name="hero"]` shortcode and a `jixel_picture('hero')` PHP
  function that render a Jixel export as `<picture>`.

Steps:

1. Copy `snippets/wordpress.php` to `wp-content/mu-plugins/jixel.php` (create
   the folder if needed). Must-use plugins are always active.
2. Upload the export folder to `wp-content/uploads/jixel/hero/` (SFTP, your
   host's file manager, or a plugin like WP File Manager).
3. In a post or page, add a *Shortcode* block with `[jixel name="hero"]`.
   Optional attributes: `alt`, `sizes`, `class`, `priority="1"`.
   In a theme: `<?php echo jixel_picture('hero', ['class' => 'cover']); ?>`

Page builders (Elementor, Divi, Gutenberg, WPBakery) all support shortcodes.

**Note:** caching/optimisation plugins that rewrite `<img>` into WebP (e.g. some
"WebP Express" setups) should be told to skip images inside `<picture>`.

## 5. Server configuration

Two things matter: the **MIME type** (`image/jxl`) and **long cache headers**
(the file names never change, so they can be cached for a year).

### nginx

Add `snippets/nginx.conf` to your `server { }` block. It sets the MIME type and
caching. It also contains an *optional* section for **content negotiation**:
serving `photo.jxl` when a page requests `photo.jpg` and the browser says it
accepts JPEG XL. That lets you use JPEG XL with an existing site without
changing its HTML, as long as you put `photo.jxl` next to `photo.jpg`.

### Apache / shared hosting

Put `snippets/apache.htaccess` in the images folder (or merge it into your
site's `.htaccess`). Same features: MIME type, cache headers, optional
negotiation with `mod_rewrite`, and the `Vary: Accept` header.

### Others

- **Caddy**: `.jxl` has the right type since 2.7.
- **IIS**: add a MIME map `.jxl → image/jxl` in `web.config`.
- **Node / Express**: `express.static` uses `mime` which knows `image/jxl`.
- **S3 / R2 / GCS**: set `Content-Type: image/jxl` on upload
  (`aws s3 cp … --content-type image/jxl`).

## 6. CDNs and caching

- With `<picture>` the URL tells the format, so any CDN works without extra
  configuration.
- If you use **content negotiation** instead, the CDN must vary its cache on
  the `Accept` header (Cloudflare: "Vary for images" or a cache rule; Fastly
  and CloudFront: include `Accept` in the cache key). Otherwise a browser
  without JPEG XL may receive a cached `.jxl`.

## 7. Build pipelines and many images

The `jixel` CLI produces the same output and can print JSON:

```bash
# Every image in a folder, one export per image
for f in src/images/*.{jpg,png}; do
  jixel web "$f" -o public/images --name "$(basename "${f%.*}")" --json
done
```

- **npm scripts**: `"images": "sh scripts/images.sh"` before `next build`.
- **GitHub Actions / CI**: install the CLI once, cache the output folder.
- **WordPress at scale**: export locally, sync `uploads/jixel/` with rsync.

## Checklist

- [ ] `.jxl` served as `image/jxl` (check the *Network* tab in DevTools)
- [ ] `width`/`height` present (no layout shift)
- [ ] Hero image exported with *Hero* (eager, high priority); others lazy
- [ ] `sizes` matches how wide the image is displayed
- [ ] Long cache headers on image files
- [ ] Alt text written
- [ ] Tested in a browser **without** JPEG XL (e.g. Firefox ESR) — the
      fallback should load
