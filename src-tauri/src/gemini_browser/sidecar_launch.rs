use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GeminiBrowserSidecarLaunch {
    Bundled { name: String },
    DevNodeScript { node: String, script: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeminiBrowserBuildProfile {
    Debug,
    Release,
}

pub(crate) const GEMINI_BROWSER_SIDECAR_NAME: &str = "gemini-browser-sidecar";

pub(crate) fn bundled_sidecar_path(executable: &Path) -> PathBuf {
    let directory = executable.parent().unwrap_or_else(|| Path::new("."));
    let filename = if cfg!(windows) {
        format!("{GEMINI_BROWSER_SIDECAR_NAME}.exe")
    } else {
        GEMINI_BROWSER_SIDECAR_NAME.to_string()
    };
    directory.join(filename)
}

pub(crate) fn bundled_sidecar_path_from_current_exe() -> std::io::Result<PathBuf> {
    std::env::current_exe().map(|executable| bundled_sidecar_path(&executable))
}

pub(crate) fn dev_sidecar_script(repo_root: &Path) -> PathBuf {
    repo_root
        .join("sidecars")
        .join("gemini-browser")
        .join("dist")
        .join("index.js")
}

pub(crate) fn resolve_launch_mode(
    build_profile: GeminiBrowserBuildProfile,
    force_dev: bool,
    force_bundled: bool,
    repo_root: &Path,
    dev_script_exists: bool,
) -> GeminiBrowserSidecarLaunch {
    if force_bundled {
        return GeminiBrowserSidecarLaunch::Bundled {
            name: GEMINI_BROWSER_SIDECAR_NAME.to_string(),
        };
    }

    if force_dev && dev_script_exists {
        return GeminiBrowserSidecarLaunch::DevNodeScript {
            node: "node".to_string(),
            script: dev_sidecar_script(repo_root),
        };
    }

    if build_profile == GeminiBrowserBuildProfile::Debug && dev_script_exists {
        return GeminiBrowserSidecarLaunch::DevNodeScript {
            node: "node".to_string(),
            script: dev_sidecar_script(repo_root),
        };
    }

    GeminiBrowserSidecarLaunch::Bundled {
        name: GEMINI_BROWSER_SIDECAR_NAME.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_launch_mode_prefers_bundled_when_forced() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            true,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }

    #[test]
    fn resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::DevNodeScript {
                node: "node".to_string(),
                script: PathBuf::from("G:/Develop/Extractum/sidecars/gemini-browser/dist/index.js")
            }
        );
    }

    #[test]
    fn resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Release,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }

    #[test]
    fn resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Release,
            true,
            false,
            Path::new("G:/Develop/Extractum"),
            true,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::DevNodeScript {
                node: "node".to_string(),
                script: PathBuf::from("G:/Develop/Extractum/sidecars/gemini-browser/dist/index.js")
            }
        );
    }

    #[test]
    fn resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent() {
        let mode = resolve_launch_mode(
            GeminiBrowserBuildProfile::Debug,
            false,
            false,
            Path::new("G:/Develop/Extractum"),
            false,
        );

        assert_eq!(
            mode,
            GeminiBrowserSidecarLaunch::Bundled {
                name: "gemini-browser-sidecar".to_string()
            }
        );
    }

    #[test]
    fn bundled_sidecar_path_is_beside_the_packaged_executable() {
        assert_eq!(
            bundled_sidecar_path(Path::new("C:/Extractum/extractum.exe")),
            PathBuf::from("C:/Extractum/gemini-browser-sidecar.exe"),
        );
    }
}
