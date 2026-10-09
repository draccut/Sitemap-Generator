use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use nix::unistd::{
    getegid, geteuid, getgid, getuid, setgid, setgroups, setuid, Gid, Uid,
};
use users::{get_group_by_name, get_user_by_name};

const USERNAME: &str = "sitemap";
const GROUPNAME: &str = "sitemap";
const HOME: &str = "/home/sitemap";

pub fn drop_p(handler_server: fn()) {
    if let Err(e) = run_server(handler_server) {
        eprintln!("Fehler beim Privilege-Drop / Start: {}", e);
        std::process::exit(1);
    }
}

fn run_server(handler_server: fn()) -> Result<(), Box<dyn std::error::Error>> {
    if geteuid() != Uid::from_raw(0) {
        eprintln!("Fehler: Dieses Programm muss als root gestartet werden.");
        handler_server();
        return Ok(());
    }
    println!("[*] Starte als root");
    let gid = match get_group_by_name(GROUPNAME) {
        Some(g) => {
            println!("[+] Gruppe '{}' existiert (GID {})", GROUPNAME, g.gid());
            Gid::from_raw(g.gid())
        }
        None => {
            println!("[-] Gruppe '{}' fehlt → wird angelegt", GROUPNAME);
            create_group(GROUPNAME)?
        }
    };
    let uid = match get_user_by_name(USERNAME) {
        Some(u) => {
            println!("[+] User '{}' existiert (UID {})", USERNAME, u.uid());
            Uid::from_raw(u.uid())
        }
        None => {
            println!("[-] User '{}' fehlt → wird angelegt", USERNAME);
            create_user(USERNAME, GROUPNAME)?
        }
    };
    setup_home(uid, gid)?;
    setgroups(&[gid])?;
    println!("[*] Wechsle zu User '{}' (UID={}, GID={})", USERNAME, uid, gid);
    setgid(gid)?;
    setuid(uid)?;
    if getuid() != uid || geteuid() != uid || getgid() != gid || getegid() != gid {
        eprintln!("Fehler: Privilege Drop fehlgeschlagen!");
        std::process::exit(1);
    }
    println!("[+] Erfolgreich als '{}' unterwegs", USERNAME);
    println!(
        "    uid={} euid={} gid={} egid={}",
        getuid(),
             geteuid(),
             getgid(),
             getegid()
    );
    handler_server();
    Ok(())
}

fn create_group(name: &str) -> Result<Gid, Box<dyn std::error::Error>> {
    let status = Command::new("groupadd").arg(name).status()?;
    if !status.success() {
        return Err(format!("groupadd für '{}' fehlgeschlagen", name).into());
    }
    let g = get_group_by_name(name).ok_or("Gruppe nach dem Anlegen nicht gefunden")?;
    Ok(Gid::from_raw(g.gid()))
}

/**
 * Home anlegen – kein Login-Shell nötig
 */
fn create_user(name: &str, group: &str) -> Result<Uid, Box<dyn std::error::Error>> {
    let status = Command::new("useradd")
    .args(["-m", "-g", group, "-s", "/usr/sbin/nologin", name])
    .status()?;
    if !status.success() {
        return Err(format!("useradd für '{}' fehlgeschlagen", name).into());
    }
    let u = get_user_by_name(name).ok_or("User nach dem Anlegen nicht gefunden")?;
    Ok(Uid::from_raw(u.uid()))
}

fn setup_home(uid: Uid, gid: Gid) -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(HOME).exists() {
        fs::create_dir_all(HOME)?;
        println!("[+] Home-Verzeichnis erstellt: {}", HOME);
    }
    let mut perms = fs::metadata(HOME)?.permissions();
    perms.set_mode(0o750);
    fs::set_permissions(HOME, perms)?;
    nix::unistd::chown(HOME, Some(uid), Some(gid))?;
    println!("[+] Home-Verzeichnis konfiguriert");
    Ok(())
}

