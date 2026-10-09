use std::sync::{Arc, OnceLock, RwLock};

/**
 * Aktueller Sitemap-Buffer als Arc<String> hinter einem RwLock.
 * - Writer tauschen nur den Arc aus (kurz write-lock).
 * - Reader clonen den Arc, nicht den ganzen String
 * - Der alte String bleibt live, solange ein Reader seinen Arc hält,
 *   danach gibt der Allocator ihn frei.
 */
static SITEMAP_XML: OnceLock<RwLock<Arc<String>>> = OnceLock::new();

fn cell() -> &'static RwLock<Arc<String>> {
    SITEMAP_XML.get_or_init(|| RwLock::new(Arc::new(String::new())))
}

/**
 * Übernimmt Box<String> legt Inhalt unveränderlich
 * in einen Arc und published ihn.
 */
pub fn set_new_char_buffer(new_box: Box<String>) {
    let new_arc = Arc::new(*new_box);
    let mut guard = cell()
        .write()
        .expect("sitemap buffer lock poisoned");
    *guard = new_arc;
}

/**
 * Thread-sicherer Snapshot auf den aktuellen Buffer.
 * Clone von Arc erhöht nur den Refcount, 
 * kein Kopieren des XML.
 */
pub fn get_char_buffer() -> Arc<String> {
    cell()
        .read()
        .expect("sitemap buffer lock poisoned")
        .clone()
}
