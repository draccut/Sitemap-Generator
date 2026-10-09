mod xml_tags;
mod xml_sitemap_gen;
mod config_template;
mod sitemap_builder;
mod heap_buffer;
mod server;
mod drop_permissions;
mod parser;

use std::time::Duration;

use drop_permissions::drop_p;
use heap_buffer::{get_char_buffer, set_new_char_buffer};
use server::run_sitemap_server;
use sitemap_builder::build_sitemap_xml;

const SITEMAP_BIND_ADDR: &str = "127.0.0.1:27000";
const UPDATE_INTERVAL_SECS: u64 = 2 * 60 * 60;

/**
 * Läuft als eigener Tokio-Task und aktualisiert periodisch die Sitemap.
 * Die CPU-lastige Erzeugung erfolgt in spawn_blocking Thread-Pool das Ergebnis
 * wird über set_new_char_buffer auf dem Heap abgelegt.
 */
async fn run_sitemap_updater() {
    let interval = Duration::from_secs(UPDATE_INTERVAL_SECS);
    loop {
        tokio::time::sleep(interval).await;
        let new_xml = tokio::task::spawn_blocking(build_sitemap_xml)
            .await
            .expect("sitemap rebuild task panicked");
        println!("Set new sitemap.xml");
        set_new_char_buffer(new_xml);
    }
}

/**
 * Baut die Tokio-Runtime, erzeugt die initiale Sitemap auf dem Heap und startet
 * Server sowie Updater-Tasks parallel.
 */
fn run_app() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(async {
        let initial = build_sitemap_xml();
        set_new_char_buffer(initial);
        debug_assert!(!get_char_buffer().is_empty());
        let updater = tokio::spawn(run_sitemap_updater());
        let server = tokio::spawn(async {
            run_sitemap_server(SITEMAP_BIND_ADDR).await;
        });
        println!("Sitemap available at http://{}/", SITEMAP_BIND_ADDR);
        let _ = tokio::join!(server, updater);
    });
}

/**
 * Prüft per `geteuid()`, ob der Prozess Root-Rechte hat.
 * Keine zusätzliche Allokation; nur ein libc-Aufruf im aktuellen Thread.
 */
#[cfg(unix)]
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/**
 * Einstiegspunkt: bei Root werden Rechte über drop_p abgegeben
 */
fn main() {
    if is_root() {
        drop_p(run_app);
    } else {
        run_app();
    }
}
