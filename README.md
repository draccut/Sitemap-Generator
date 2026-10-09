# Sitemap Generator

## Projektbegründung

Sitemaps mit Timestamps der letzten Änderung von Dateien helfen Google,
nur Daten zu crawlen, die der Suchmaschine noch nicht bekannt sind.
Dadurch werden mehr neue Inhalte potenziell in den Index aufgenommen.

Werden Daten wiederholt gecrawlt, obwohl sich nichts geändert hat,
verbrauchen diese Anfragen Ressourcen, die sonst für die Erfassung
neuer Inhalte zur Verfügung stünden. Ein präzises `lastmod`-Feld in
der Sitemap reduziert unnötige Crawls und steigert die Effizienz
sowohl auf Seiten des Crawlers als auch des eigenen Servers.

https://ixxc.de/sitemap.xml

----

## Drei Arten der URL-/Datei-Zuordnung

Die Konfiguration erfolgt über die XML-Datei `sitemap_config.xml`.
Es gibt drei unterschiedliche Mechanismen, wie URLs in die Sitemap gelangen:

### 1. Statische Links (`static_links`)

Feste URLs, die direkt in der Konfiguration hinterlegt werden.
Jeder Eintrag besitzt ein `interval`, das den `lastmod`-Wert steuert:

| Wert | Bedeutung |
|------|-----------|
| `TODAY` | Heutiger Tagesbeginn (00:00:00 lokal) |
| *leer* | Fallback-Default-Datum |
| Timestamp | Fester Zeitpunkt im Format `%Y-%m-%dT%H:%M:%S%:z` |

Ungültige Intervalle werden mit einer Fehlermeldung übersprungen
und erscheinen nicht in der Sitemap.

### 2. Dynamische Links (`dynamic_links`)

Pfade mit einem `{}`-Platzhalter, der durch eine fortlaufende Nummer
ersetzt wird. Der Nummernbereich (`nr_from` … `nr_to`) kommt aus der
XML-Konfiguration.

Für jede generierte Datei gilt:

- Der **Dateiinhalt** wird als Sitemap-URL (`loc`) verwendet.
- Der **Zeitstempel der letzten Änderung** (mtime) der Datei wird als `lastmod` übernommen.

Damit lassen sich z. B. nummerierte Backend-Dateien abbilden,
ohne jede URL einzeln zu pflegen.

### 3. Rekursive Verzeichnisse (`recursiv_dir`)

Ein Dateisystempfad wird rekursiv durchlaufen. Jede gefundene Datei
wird auf eine URL unter der konfigurierten `base_domain` gemappt.

Beispiel:

| Feld | Wert |
|------|------|
| `path` | `/var/www/html/index_folder` |
| `base_domain` | `https://example.com/index_folder` |

Eine Datei `/var/www/html/index_folder/docs/guide.html` wird zu
`https://example.com/index_folder/docs/guide.html`.
`lastmod` stammt wiederum aus der mtime der Datei.

So lässt sich ein gesamtes Frontend- oder Backend-Verzeichnis
indexieren, ohne einzelne Einträge zu konfigurieren.

----

## Architektur

Der Dienst arbeitet mit **zwei Threads**:

1. **Builder-Thread (asynchron)**
   Liest die XML-Konfiguration, die Metadaten der Dateien und die
   URL-Dateien. Daraus berechnet er die vollständige Sitemap.
   Das Ergebnis wird dauerhaft auf dem **Heap** in einem
   **Read-Only-Buffer** abgelegt, den der Server-Thread nur lesend
   nutzt. Dadurch entfallen Locks und Race Conditions beim
   Ausliefern der Sitemap.

2. **Server-Thread**
   Liefert den Read-Only-Buffer über HTTP aus. Die eigentliche
   TLS-Verschlüsselung übernimmt ein vorgeschalteter
   **Reverse-Proxy** (z. B. nginx, Caddy, Apache). Der
   Sitemap-Dienst selbst spricht nur Klartext-HTTP und bleibt
   bewusst schlank.

### Rechte & Isolation

Wird der Prozess als **root** gestartet, legt er automatisch einen
Systembenutzer und eine Gruppe `sitemap` an. Anschließend wechselt
sowohl der Server als auch der Builder-Thread in diesen
unprivilegierten Kontext. Dadurch laufen Konfigurationslesen,
Dateisystem-Walks und die HTTP-Auslieferung ohne root-Rechte.

----

## Aufbau der Module

| Modul | Aufgabe |
|-------|---------|
| `parser.rs` | Deserialisiert `sitemap_config.xml`, validiert Intervalle, stellt Getter bereit |
| `config_template.rs` | Öffentliche Wrapper (`get_static_links`, `get_dynamic_links`, `get_recursiv_dir`) |
| `sitemap_builder.rs` | Sammelt Einträge aus allen drei Quellen, löst Timestamps auf, erzeugt die XML |
| `xml_sitemap_gen` | Formatiert die Eintragsliste als gültiges Sitemap-XML |

Speicherverwaltung folgt Rusts Ownership-Modell: alle
`(loc, lastmod)`-Paare leben als owned `String`s in einem `Vec`.
Die fertige XML-Zeichenkette wird als `Box<String>` an den Aufrufer
übergeben und beim Drop automatisch freigegeben es gibt keine
manuellen `free`-Aufrufe.


