use std::path::PathBuf;

pub fn default_binary_path() -> PathBuf {
    let exe_name = if cfg!(windows) {
        "maolan-plugin-host.exe"
    } else {
        "maolan-plugin-host"
    };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(exe_name);
            if candidate.exists() {
                return candidate;
            }
        }
    }
    PathBuf::from(exe_name)
}
