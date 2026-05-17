use sysinfo::{ProcessRefreshKind, RefreshKind, System};

const FLATPAK_APP_ID: &str = "app.zen_browser.zen";
const KNOWN_BINARIES: &[&str] = &["zen", "zen-bin", "zen-browser"];

pub fn zen_is_running() -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new()),
    );

    sys.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_lowercase();
        if KNOWN_BINARIES.contains(&name.as_str()) {
            return true;
        }
        p.cmd()
            .iter()
            .any(|arg| arg.to_string_lossy().contains(FLATPAK_APP_ID))
    })
}
