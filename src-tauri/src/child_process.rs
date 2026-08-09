use tokio::process::Command;

#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn hide_console_window(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(test)]
mod tests {
    use super::CREATE_NO_WINDOW;

    #[test]
    fn create_no_window_matches_win32_process_creation_flags() {
        assert_eq!(CREATE_NO_WINDOW, 0x0800_0000);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn gemini_sidecar_launches_hide_console_window_on_windows() {
        let script = "Add-Type -Name Native -Namespace Extractum -MemberDefinition '[DllImport(\"kernel32.dll\")] public static extern System.IntPtr GetConsoleWindow();'; [Console]::Out.Write([Extractum.Native]::GetConsoleWindow().ToInt64())";
        let mut command = tokio::process::Command::new("powershell.exe");
        command.args(["-NoProfile", "-Command", script]);
        crate::gemini_browser::configure_sidecar_command(&mut command);
        let output = command
            .output()
            .await
            .expect("launch configured sidecar fixture");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "0");
        assert_eq!(CREATE_NO_WINDOW, 0x0800_0000);
    }

    #[cfg(not(windows))]
    #[test]
    fn gemini_sidecar_launches_hide_console_window_on_windows() {
        assert_eq!(CREATE_NO_WINDOW, 0x0800_0000);
    }
}
