/// Cross-platform active window/application detection

#[cfg(target_os = "macos")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

#[cfg(target_os = "windows")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;
    // PowerShell one-liner to get foreground window process name
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Process -Id (Get-CimInstance Win32_Process -Filter \"ProcessId = $((Add-Type -MemberDefinition '[DllImport(\"user32.dll\")] public static extern IntPtr GetForegroundWindow();[DllImport(\"user32.dll\")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);' -Name User32 -Namespace Win32 -PassThru)::GetWindowThreadProcessId([Win32.User32]::GetForegroundWindow(), [ref]($pid = 0)) | Out-Null; $pid)\".ProcessId\")).ProcessName",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn get_active_app_name() -> Option<String> {
    use std::process::Command;

    // Try xdotool first (X11)
    if let Ok(output) = Command::new("xdotool")
        .args(["getactivewindow", "getwindowpid"])
        .output()
    {
        if output.status.success() {
            let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
                let name = cmdline.trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }

    // Fallback: try swaymsg for Wayland/Sway
    if let Ok(output) = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
    {
        if output.status.success() {
            if let Ok(tree) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(name) = find_focused_app_name(&tree) {
                    return Some(name);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn find_focused_app_name(node: &serde_json::Value) -> Option<String> {
    if node.get("focused")?.as_bool()? {
        return node
            .get("app_id")
            .and_then(|v| v.as_str())
            .or_else(|| {
                node.get("window_properties")
                    .and_then(|wp| wp.get("class"))
                    .and_then(|v| v.as_str())
            })
            .map(|s| s.to_string());
    }

    if let Some(nodes) = node.get("nodes").and_then(|n| n.as_array()) {
        for child in nodes {
            if let Some(name) = find_focused_app_name(child) {
                return Some(name);
            }
        }
    }
    if let Some(nodes) = node.get("floating_nodes").and_then(|n| n.as_array()) {
        for child in nodes {
            if let Some(name) = find_focused_app_name(child) {
                return Some(name);
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn get_active_app_name() -> Option<String> {
    None
}
