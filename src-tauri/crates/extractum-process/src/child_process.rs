use tokio::process::Command;

#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn hide_console_window(command: &mut Command) -> &mut Command {
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
}
