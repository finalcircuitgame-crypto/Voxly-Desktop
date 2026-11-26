use std::path::Path;

#[cfg(windows)]
pub fn get_foreground_game() -> Option<String> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let window = GetForegroundWindow();
        if !window.is_null() {
            let mut process_id = 0u32;
            if GetWindowThreadProcessId(window, &mut process_id) != 0 && process_id != 0 {
                let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
                if !process.is_null() {
                    let mut buffer = [0u16; 1024];
                    let mut length = buffer.len() as u32;
                    let success = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
                    CloseHandle(process);

                    if success != 0 && length > 0 {
                        let path = std::ffi::OsString::from_wide(&buffer[..length as usize]);
                        let executable = Path::new(&path)
                            .file_name()
                            .and_then(|name| name.to_str())?
                            .to_ascii_lowercase();
                        if let Some(game) = game_name(&executable) {
                            return Some(game.to_owned());
                        }
                    }
                }
            }
        }
    }

    // Keep activity alive when the player switches back to Voxly or another
    // window while the game remains running in the background.
    get_running_game()
}

#[cfg(windows)]
fn get_running_game() -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = None;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let length = entry.szExeFile.iter().position(|unit| *unit == 0)
                    .unwrap_or(entry.szExeFile.len());
                let executable = String::from_utf16_lossy(&entry.szExeFile[..length])
                    .to_ascii_lowercase();

                if let Some(game) = game_name(&executable) {
                    found = Some(game.to_owned());
                    break;
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
pub fn get_foreground_game() -> Option<String> {
    None
}

fn game_name(executable: &str) -> Option<&'static str> {
    match executable {
        // Roblox is recognized as a fallback; the backend preserves richer SDK
        // activity whenever the Roblox integration is actively publishing.
        "robloxplayerbeta.exe" | "robloxplayerlauncher.exe" => Some("Roblox"),

        // Call of Duty HQ / Warzone / recent releases.
        "cod.exe" | "cod22-cod.exe" | "cod23-cod.exe" | "cod24-cod.exe"
        | "cod25-cod.exe" | "cod26-cod.exe" => Some("Call of Duty"),

        "fortniteclient-win64-shipping.exe" => Some("Fortnite"),
        "valorant-win64-shipping.exe" => Some("VALORANT"),
        "gta5.exe" | "gta5_enhanced.exe" => Some("Grand Theft Auto V"),
        "r5apex.exe" => Some("Apex Legends"),
        "overwatch.exe" => Some("Overwatch 2"),
        "rocketleague.exe" => Some("Rocket League"),
        "cs2.exe" => Some("Counter-Strike 2"),
        "rainbow6.exe" | "rainbowsix.exe" => Some("Rainbow Six Siege"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::game_name;

    #[test]
    fn recognizes_call_of_duty_processes() {
        assert_eq!(game_name("cod.exe"), Some("Call of Duty"));
        assert_eq!(game_name("cod26-cod.exe"), Some("Call of Duty"));
    }

    #[test]
    fn recognizes_roblox_for_sdk_fallback() {
        assert_eq!(game_name("robloxplayerbeta.exe"), Some("Roblox"));
    }

    #[test]
    fn ignores_unrecognized_processes() {
        assert_eq!(game_name("explorer.exe"), None);
    }
}
