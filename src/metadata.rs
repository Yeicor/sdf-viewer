use shadow_rs::shadow;
use tracing::info;

shadow!(build);

#[allow(dead_code)] // Allow auto-generated code containing unused build metadata
pub(crate) fn log_version_info() {
    info!("{}", short_version_info());
    info!("Build date: {} ({})", build::BUILD_TIME_2822, build::BUILD_RUST_CHANNEL);
}

#[allow(dead_code)] // Allow auto-generated code containing unused build metadata
pub(crate) fn short_version_info() -> String {
    format!("{} {} ({}@{}{})", build::PROJECT_NAME, build::PKG_VERSION, build::BRANCH, build::SHORT_COMMIT,
            if shadow_rs::git_clean() { "" } else { "+dirty" })
}

#[allow(dead_code)] // Allow auto-generated code containing unused build metadata
pub(crate) fn short_version_info_is_ours(s: &str) -> bool {
    s.contains(&build::PROJECT_NAME.to_string())
}