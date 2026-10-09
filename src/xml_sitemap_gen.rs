use crate::xml_tags::{SITEMAP_HEADER, SITEMAP_URL_ENTRY, SITEMAP_FOOTER};

pub fn generate_sitemap_from_entries(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str(SITEMAP_HEADER);
    for (loc, lastmod) in entries {
        let entry = SITEMAP_URL_ENTRY
            .replace("{loc}", loc)
            .replace("{lastmod}", lastmod);
        out.push_str(&entry);
    }
    out.push_str(SITEMAP_FOOTER);
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_generates_three_dummy_entries_and_contains_loc() {
        let entries = vec![
            ("https://www.example.com/foo.html".to_string(), "2022-06-04".to_string()),
            ("https://ixxc.de/register".to_string(), "2026-08-01".to_string()),
            ("https://ixxc.de/pubstream".to_string(), "2026-08-30".to_string()),
        ];
        let sitemap = generate_sitemap_from_entries(&entries);
        println!("{}", sitemap);
        assert!(sitemap.contains("<urlset"));
        assert!(sitemap.contains("https://www.example.com/foo.html"));
        assert!(sitemap.matches("<url>").count() >= 3);
    }
}

