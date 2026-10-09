pub const SITEMAP_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#;

pub const SITEMAP_URL_ENTRY: &str = r#"  <url>
    <loc>{loc}</loc>
    <lastmod>{lastmod}</lastmod>
  </url>
"#;

pub const SITEMAP_FOOTER: &str = r#"</urlset>
"#;

