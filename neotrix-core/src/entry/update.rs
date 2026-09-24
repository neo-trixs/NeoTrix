//! update — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::info;

pub fn run_update(check_only: bool) {
    println!("{} v{}", info("NeoTrix Update"), env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "self-update")]
    {
        use self_update::cargo_crate_version;
        if check_only {
            println!("{}", info("Checking for updates..."));
            match self_update::backends::github::Update::configure()
                .repo_owner("neotrix")
                .repo_name("neotrix")
                .bin_name("neotrix")
                .show_download_progress(true)
                .current_version(cargo_crate_version!())
                .build()
            {
                Ok(updater) => match updater.get_latest_release() {
                    Ok(release) => {
                        println!("{} {}", info("Current version:"), env!("CARGO_PKG_VERSION"));
                        println!("{} {}", info("Latest version:"), release.version);
                        if release.version != cargo_crate_version!() {
                            println!(
                                "{}",
                                success("✅ Update available! Run `neotrix update` to install.")
                            );
                        } else {
                            println!("{}", success("✅ You have the latest version."));
                        }
                    }
                    Err(e) => eprintln!("{}: {}", err("Check failed"), e),
                },
                Err(e) => eprintln!("{}: {}", err("Update config failed"), e),
            }
        } else {
            println!("{}", info("Updating NeoTrix..."));
            match self_update::backends::github::Update::configure()
                .repo_owner("neotrix")
                .repo_name("neotrix")
                .bin_name("neotrix")
                .show_download_progress(true)
                .current_version(cargo_crate_version!())
                .build()
            {
                Ok(updater) => match updater.update() {
                    Ok(status) => {
                        println!("{} {}", success("✅ Update complete:"), status.version());
                    }
                    Err(e) => eprintln!("{}: {}", err("Update failed"), e),
                },
                Err(e) => eprintln!("{}: {}", err("Update config failed"), e),
            }
        }
    }
    #[cfg(not(feature = "self-update"))]
    {
        let _ = check_only;
        println!("{}", info("Self-update is not enabled in this build."));
        println!(
            "{}",
            info("Build with --features self-update or use your package manager.")
        );
    }
}
