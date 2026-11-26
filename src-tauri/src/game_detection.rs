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
        if window.is_null() {
            return None;
        }

        let mut process_id = 0u32;
        if GetWindowThreadProcessId(window, &mut process_id) == 0 || process_id == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return None;
        }

        let mut buffer = [0u16; 1024];
        let mut length = buffer.len() as u32;
        let success = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(process);

        if success == 0 || length == 0 {
            return None;
        }

        let path = std::ffi::OsString::from_wide(&buffer[..length as usize]);
        let executable = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())?
            .to_ascii_lowercase();

        game_name(&executable).map(str::to_owned)
    }
}

#[cfg(not(windows))]
pub fn get_foreground_game() -> Option<String> {
    None
}

fn game_name(executable: &str) -> Option<&'static str> {
    match executable {
        // Roblox is intentionally left to the Voxly Roblox SDK, which can
        // provide richer experience-specific details than process detection.
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
    fn leaves_roblox_to_the_sdk() {
        assert_eq!(game_name("robloxplayerbeta.exe"), Some("Roblox"));
    }

    #[test]
    fn ignores_unrecognized_processes() {
        assert_eq!(game_name("explorer.exe"), None);
    }
}
