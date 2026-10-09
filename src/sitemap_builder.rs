use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config_template::{get_dynamic_links, get_recursiv_dir, get_static_links};
use crate::xml_sitemap_gen::generate_sitemap_from_entries;

pub const DEFAULT_LASTMOD: &str = "2026-09-15T13:45:00+02:00";

/**
 * Formatiert einen SystemTime als ISO-8601-String mit Offset
 * %Y-%m-%dT%H:%M:%S%:z
 */
fn format_mtime(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/**
 * Liefert den heutigen Tagesbeginn (00:00:00) als ISO-8601-String
 * mit lokalem Offset. Wird für Interval TODAY verwendet.
 */
fn today_midnight_iso() -> String {
    let now: DateTime<Local> = Local::now();
    let date = now.date_naive();
    let naive_mid: NaiveDateTime = date.and_hms_opt(0, 0, 0).unwrap();
    let local_mid = Local
    .from_local_datetime(&naive_mid)
    .single()
    .unwrap_or(now);
    local_mid.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

fn normalize_or_resolve_ts(_loc: &str, ts: &str) -> String {
    if ts == "TODAY" {
        today_midnight_iso()
    } else if ts.is_empty() {
        DEFAULT_LASTMOD.to_string()
    } else {
        ts.to_string()
    }
}

fn file_timestamp_iso(path: &Path) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let time = meta.created().or_else(|_| meta.modified()).ok()?;
    Some(format_mtime(time))
}

/**
 * Liest den Inhalt einer Datei als Sitemap-URL loc und ermittelt
 * den lastmod-Timestamp aus den Datei-Metadaten.
 * Gibt None zurück, wenn die Datei nicht existiert, leer ist oder
 * nicht gelesen werden kann. Dadurch wird der Eintrag übersprungen.
 */
fn try_entry_from_file(path: &str) -> Option<(String, String)> {
    let p = Path::new(path);
    if !p.is_file() {
        return None;
    }
    let content = fs::read_to_string(p).ok()?;
    let loc = content.trim().to_string();
    if loc.is_empty() {
        return None;
    }
    let lastmod = file_timestamp_iso(p).unwrap_or_else(|| DEFAULT_LASTMOD.to_string());
    Some((loc, lastmod))
}

/**
 * Sammelt dynamische Einträge. Für jeden XML-Eintrag path, nr_from, nr_to
 * wird der Platzhalter {} im Pfad durch jede Zahl im Bereich nr_from bis nr_to ersetzt.
 * Existiert die resultierende Datei, wird ihr Inhalt als URL und ihre
 * mtime als lastmod übernommen.
 */
fn collect_dynamic_entries() -> Vec<(String, String)> {
    let dyn_links = get_dynamic_links();
    let mut entries = Vec::new();
    for (path_template, nr_from, nr_to) in dyn_links {
        let parts: Vec<&str> = path_template.split("{}").collect();
        if parts.len() != 2 {
            eprintln!(
                "Dynamic path '{}' contains no '{{}}' placeholder – skipping.",
                path_template
            );
            continue;
        }
        for index in nr_from..=nr_to {
            let numbered_path = format!("{}{}{}", parts[0], index, parts[1]);
            if let Some(page_content) = try_entry_from_file(&numbered_path) {
                entries.push(page_content);
            }
        }
    }
    entries
}

/**
 * Durchläuft dir und alle Unterverzeichnisse. Für jede reguläre Datei
 * wird der relative Pfad zu base_fs ermittelt, an base_url angehängt
 * und zusammen mit der mtime in out geschrieben.
 * Ownership: die erzeugten Strings gehören dem übergebenen Vec
 */
fn walk_dir_recursive(dir: &Path, base_fs: &Path, base_url: &str, out: &mut Vec<(String, String)>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Cannot read directory '{}': {e}", dir.display());
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir_recursive(&path, base_fs, base_url, out);
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base_fs) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                let url = if base_url.ends_with('/') {
                    format!("{}{}", base_url, rel_str)
                } else {
                    format!("{}/{}", base_url, rel_str)
                };
                let lastmod = file_timestamp_iso(&path).unwrap_or_else(|| DEFAULT_LASTMOD.to_string());
                out.push((url, lastmod));
            }
        }
    }
}

/**
 * Sammelt Einträge aus den rekursiven Verzeichnis-Konfigurationen.
 * Für jeden XML-Eintrag fs_path, base_domain wird das Dateisystem
 * rekursiv durchlaufen. Jede gefundene Datei wird auf eine
 * URL unter base_domain gemappt lastmod kommt aus der mtime Metadaten der Datei.
 */
fn collect_recursive_entries() -> Vec<(String, String)> {
    let dirs = get_recursiv_dir();
    let mut entries = Vec::new();
    for (fs_path, base_domain) in dirs {
        let base = PathBuf::from(&fs_path);
        if !base.is_dir() {
            eprintln!(
                "Recursive path '{}' is not a directory – skipping.",
                fs_path
            );
            continue;
        }
        walk_dir_recursive(&base, &base, &base_domain, &mut entries);
    }
    entries
}

/**
 * Baut die vollständige Liste aller Sitemap-Einträge:
 * 1. Statische Links Interval: konkreter Timestamp
 * 2. Dynamische Links XML-Nummernbereich + Datei-Inhalt/mtime
 * 3. Rekursive Verzeichnisse Dateibaum: URL + mtime
 * Der zurückgegebene Vec besitzt alle Strings der Speicher wird
 * automatisch freigegeben, sobald der Vec den Scope verlässt.
 */
fn build_resolved_entries() -> Vec<(String, String)> {
    let static_paths = get_static_links();
    let mut vector: Vec<(String, String)> =
    Vec::with_capacity(static_paths.len() + 0x50);
    for (loc, ts) in static_paths.into_iter() {
        let resolved = normalize_or_resolve_ts(&loc, &ts);
        vector.push((loc, resolved));
    }
    vector.extend(collect_dynamic_entries());
    vector.extend(collect_recursive_entries());
    vector
}

/**
 * Öffentliche Einstiegsfunktion: erzeugt das fertige Sitemap-XML.
 * Die zurückgegebene `Box<String>` übergibt Ownership der XML-Zeichenkette
 * an den Aufrufer. Sobald die Box gedroppt wird, wird der Speicher
 * automatisch freigegeben
 */
pub fn build_sitemap_xml() -> Box<String> {
    let entries = build_resolved_entries();
    Box::new(generate_sitemap_from_entries(&entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * Prüft die drei erlaubten Intervall-Auflösungen:
     * fester Timestamp, TODAY und leerer String (Default).
     */
    #[test]
    fn test_normalize_today_and_default() {
        assert_eq!(normalize_or_resolve_ts("https://ixxc.de/lang", "2026-08-31T14:14:00+02:00"), "2026-08-31T14:14:00+02:00".to_string());
        let t = normalize_or_resolve_ts("https://ixxc.de/register", "TODAY");
        assert!(t.contains("T00:00:00"));
        assert_eq!(normalize_or_resolve_ts("https://ixxc.de/", ""), DEFAULT_LASTMOD.to_string());
    }

    /**
     * format_mtime muss einen gültigen ISO-String mit Offset erzeugen.
     */
    #[test]
    fn test_format_mtime_contains_offset() {
        let now = std::time::SystemTime::now();
        let s = format_mtime(now);
        assert!(s.contains('T'));
        assert!(s.contains('+') || s.contains('-'));
    }

    /**
     * today_midnight_iso muss auf 00:00:00 enden.
     */
    #[test]
    fn test_today_midnight_iso() {
        let s = today_midnight_iso();
        assert!(s.contains("T00:00:00"));
    }
}
