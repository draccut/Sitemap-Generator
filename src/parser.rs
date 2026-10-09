#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use std::path::Path;
use std::process;

use serde::{Deserialize, Serialize};

#[cfg(not(test))]
pub const CONFIG_FILE: &str = "sitemap_config.xml";

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct StaticLink {
    pub url: String,
    pub interval: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DynamicLink {
    pub path: String,
    pub nr_from: u64,
    pub nr_to: u64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Directory {
    pub path: String,
    pub base_domain: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LinkConfiguration {
    #[serde(rename = "static_links", default)]
    pub static_links: Vec<StaticLink>,
    #[serde(rename = "dynamic_links", default)]
    pub dynamic_links: Vec<DynamicLink>,
    #[serde(rename = "recursiv_dir", default)]
    pub recursiv_dir: Vec<Directory>,
}

impl LinkConfiguration {

    /**
     * Lädt die Link-Konfiguration aus der XML-Datei
     */
    pub fn load() -> Self {
        let xml = get_xml_content();
        parse_xml(&xml)
    }

    /**
     * Liefert alle gültigen statischen Links als Vektor von Vec(url, interval)
     */
    pub fn get_static_links(&self) -> Vec<(String, String)> {
        self.static_links
        .iter()
        .filter_map(|l| {
            if is_valid_interval(&l.interval) {
                Some((l.url.clone(), l.interval.clone()))
            } else {
                eprintln!(
                    "Invalid interval '{}' for URL '{}'. Skipping entry.",
                    l.interval, l.url
                );
                None
            }
        })
        .collect()
    }

    /**
     * Liefert alle dynamischen Links als Vektor von Vec(path, nr_from, nr_to)
     */
    pub fn get_dynamic_links(&self) -> Vec<(String, u64, u64)> {
        self.dynamic_links
        .iter()
        .map(|l| (l.path.clone(), l.nr_from, l.nr_to))
        .collect()
    }

    /**
     * Liefert alle rekursiven Verzeichnis-Einträge als Vektor von Vec(path, base_domain)
     */
    pub fn get_recursiv_dir(&self) -> Vec<(String, String)> {
        self.recursiv_dir
        .iter()
        .map(|d| (d.path.clone(), d.base_domain.clone()))
        .collect()
    }
}

fn is_valid_interval(interval: &str) -> bool {
    if interval.is_empty() || interval == "TODAY" {
        return true;
    }
    chrono::DateTime::parse_from_str(interval, "%Y-%m-%dT%H:%M:%S%:z").is_ok()
}

fn parse_xml(xml: &str) -> LinkConfiguration {
    match quick_xml::de::from_str(xml) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to parse sitemap config XML: {e}");
            process::exit(1);
        }
    }
}

/**
 * Produktion: XML-Inhalt aus der Datei sitemap_config.xml lesen
 */
#[cfg(not(test))]
fn get_xml_content() -> String {
    let path = Path::new(CONFIG_FILE);
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {e}", CONFIG_FILE);
            process::exit(1);
        }
    }
}

/**
 * Test: fester XML-Inhalt mit allen drei Knotenarten je zwei Einträge
 */
#[cfg(test)]
fn get_xml_content() -> String {
    TEST_XML.to_string()
}

#[cfg(test)]
const TEST_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LinkConfiguration>
<static_links>
<url>https://example.com/page1</url>
<interval>daily</interval>
</static_links>
<static_links>
<url>https://example.com/page2</url>
<interval>weekly</interval>
</static_links>
<dynamic_links>
<path>/products/</path>
<nr_from>1</nr_from>
<nr_to>100</nr_to>
</dynamic_links>
<dynamic_links>
<path>/users/</path>
<nr_from>10</nr_from>
<nr_to>50</nr_to>
</dynamic_links>
<recursiv_dir>
<path>/var/www/docs</path>
<base_domain>https://docs.example.com</base_domain>
</recursiv_dir>
<recursiv_dir>
<path>/var/www/blog</path>
<base_domain>https://blog.example.com</base_domain>
</recursiv_dir>
</LinkConfiguration>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded() -> LinkConfiguration {
        LinkConfiguration::load()
    }

    #[test]
    fn test_static_links_parsing_filters_invalid() {
        let cfg = loaded();
        let links = cfg.get_static_links();
        assert_eq!(
            links.len(),
                   0,
                   "ungültige Intervalle müssen herausgefiltert werden"
        );
    }

    #[test]
    fn test_static_links_valid_intervals() {
        let cfg = LinkConfiguration {
            static_links: vec![
                StaticLink {
                    url: "https://example.com/today".to_string(),
                    interval: "TODAY".to_string(),
                },
                StaticLink {
                    url: "https://example.com/empty".to_string(),
                    interval: "".to_string(),
                },
                StaticLink {
                    url: "https://example.com/ts".to_string(),
                    interval: "2024-05-15T12:30:00+02:00".to_string(),
                },
                StaticLink {
                    url: "https://example.com/bad".to_string(),
                    interval: "daily".to_string(),
                },
            ],
            dynamic_links: vec![],
            recursiv_dir: vec![],
        };

        let links = cfg.get_static_links();
        assert_eq!(links.len(), 3, "genau drei gültige Einträge erwartet");
        assert_eq!(
            links[0],
            ("https://example.com/today".to_string(), "TODAY".to_string())
        );
        assert_eq!(
            links[1],
            ("https://example.com/empty".to_string(), "".to_string())
        );
        assert_eq!(
            links[2],
            (
                "https://example.com/ts".to_string(),
             "2024-05-15T12:30:00+02:00".to_string()
            )
        );
    }

    #[test]
    fn test_dynamic_links_parsing() {
        let cfg = loaded();
        let links = cfg.get_dynamic_links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], ("/products/".to_string(), 1, 100));
        assert_eq!(links[1], ("/users/".to_string(), 10, 50));
    }

    #[test]
    fn test_recursiv_dir_parsing() {
        let cfg = loaded();
        let dirs = cfg.get_recursiv_dir();
        assert_eq!(dirs.len(), 2);
        assert_eq!(
            dirs[0],
            (
                "/var/www/docs".to_string(),
             "https://docs.example.com".to_string()
            )
        );
        assert_eq!(
            dirs[1],
            (
                "/var/www/blog".to_string(),
             "https://blog.example.com".to_string()
            )
        );
    }

    /**
     * Stellt sicher, dass alle drei Knotenarten vorhanden sind und
     * die Roh-Structs korrekt deserialisiert wurden.
     */
    #[test]
    fn test_all_node_types_present() {
        let cfg = loaded();
        assert_eq!(cfg.static_links.len(), 2);
        assert_eq!(cfg.dynamic_links.len(), 2);
        assert_eq!(cfg.recursiv_dir.len(), 2);
        assert_eq!(
            cfg.static_links[0],
            StaticLink {
                url: "https://example.com/page1".to_string(),
                   interval: "daily".to_string(),
            }
        );
        assert_eq!(
            cfg.dynamic_links[1],
            DynamicLink {
                path: "/users/".to_string(),
                   nr_from: 10,
                   nr_to: 50,
            }
        );
        assert_eq!(
            cfg.recursiv_dir[0],
            Directory {
                path: "/var/www/docs".to_string(),
                   base_domain: "https://docs.example.com".to_string(),
            }
        );
    }

    #[test]
    fn test_is_valid_interval() {
        assert!(is_valid_interval("TODAY"));
        assert!(is_valid_interval(""));
        assert!(is_valid_interval("2024-05-15T12:30:00+02:00"));
        assert!(is_valid_interval("2023-01-01T00:00:00+00:00"));
        assert!(!is_valid_interval("daily"));
        assert!(!is_valid_interval("weekly"));
        assert!(!is_valid_interval("2024-05-15"));
        assert!(!is_valid_interval("15.05.2024T12:30:00"));
        assert!(!is_valid_interval("now"));
    }
}

