use crate::parser::LinkConfiguration;

/**
 * get_static_links: Eine Fixe URL die entweder ein Manuelles Datum erhält
 * oder mit TODAY immer das des heutiges Tages oder ein Default Datum
 * falls der Tag lehr ist. Ungültige Einträge werden mit einer
 * Fehlermeldung übersprungen.
 *
 * Rückgabe: `Vec<(url, interval)>`
 *
 * Gültige Werte:
 * TODAY: aktuelles Datum
 * empty tag: Default-Datum
 * Timestamp im Format %Y-%m-%dT%H:%M:%S%:z z.B: 2026-09-14T21:53:37+02:00
 */
pub fn get_static_links() -> Vec<(String, String)> {
    LinkConfiguration::load().get_static_links()
}

/**
 * get_dynamic_links: Hat im Tag path einen Platzhalter der jeweils für
 * einen unsigned interger u64 steht, dies ist für Files die nur eine
 * URL enthalten der Zeitpunkt resultiert aus den Metadaten der Datei.
 *
 * Rückgabe: Vec<(path, nr_from, nr_to)>
 */
pub fn get_dynamic_links() -> Vec<(String, u64, u64)> {
    LinkConfiguration::load().get_dynamic_links()
}

/**
 * get_recursiv_dir: path ist eine Verzeichnistruktur von Frontend Datein
 * die so auch als Endpunkte existieren. base_domain die basis URL auf
 * die diese Datein gemappt weden also wenn im Apache root index_folder
 * liegt steht bei path /var/www/html/index_folder und bei base_url
 * https://example.com/index_folder es wird rekursiv durch das
 * verzeichnis gegangen und danach gesucht.
 *
 * Rückgabe: Vec<(path, base_domain)>
 */
pub fn get_recursiv_dir() -> Vec<(String, String)> {
    LinkConfiguration::load().get_recursiv_dir()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_static_links_returns_vec() {
        let links = get_static_links();
        assert!(links.is_empty() || !links.is_empty());
    }

    #[test]
    fn test_get_dynamic_links_returns_vec() {
        let links = get_dynamic_links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].0, "/products/");
        assert_eq!(links[1].0, "/users/");
    }

    #[test]
    fn test_get_recursiv_dir_returns_vec() {
        let dirs = get_recursiv_dir();
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0].0, "/var/www/docs");
        assert_eq!(dirs[1].0, "/var/www/blog");
    }
}
