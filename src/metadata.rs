use shadow_rs::shadow;
use tracing::info;

shadow!(build);

pub(crate) fn log_version_info() {
    info!("{} {} ({}@{}{})", build::PROJECT_NAME, build::PKG_VERSION, build::BRANCH, build::SHORT_COMMIT,
        if shadow_rs::git_clean() {""} else {"+dirty"});
    info!("Build date: {} ({})", build::BUILD_TIME_2822, build::BUILD_RUST_CHANNEL);
}
