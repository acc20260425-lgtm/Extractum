use crate::error::{AppError, AppResult};

pub(crate) const MCP_BIND_ADDRESS: &str = "127.0.0.1";

pub(crate) fn require_local_dev_command(bind_address: &str) -> AppResult<()> {
    if cfg!(dev) && bind_address == MCP_BIND_ADDRESS {
        Ok(())
    } else {
        Err(AppError::validation(
            "development fixture commands require a localhost development build",
        ))
    }
}

pub(crate) fn production_devtools_allowed(csp_verified: bool) -> bool {
    cfg!(feature = "csp-verification") && csp_verified
}
