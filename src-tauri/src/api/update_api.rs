use serde::Deserialize;

use crate::api::message::Message;

#[app_macros::ipc_type]
pub struct UpdateInfo {
    pub update_available: bool,
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
}

#[taurpc::procedures(path = "updates")]
pub trait UpdateApi {
    async fn check_for_update(current_version: String) -> Message<UpdateInfo>;
}

#[derive(Clone)]
pub struct UpdateApiImpl;

fn no_update() -> Message<UpdateInfo> {
    Message::Success(UpdateInfo {
        update_available: false,
        latest_version: String::new(),
        release_url: String::new(),
    })
}

#[taurpc::resolvers]
impl UpdateApi for UpdateApiImpl {
    async fn check_for_update(self, current_version: String) -> Message<UpdateInfo> {
        let result = fetch_latest_release().await;

        let release = match result {
            Ok(r) => r,
            Err(_) => return no_update(),
        };

        if release.draft || release.prerelease {
            return no_update();
        }

        let tag = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);

        let latest = match semver::Version::parse(tag) {
            Ok(v) => v,
            Err(_) => return no_update(),
        };

        let current = match semver::Version::parse(&current_version) {
            Ok(v) => v,
            Err(_) => return no_update(),
        };

        if latest > current {
            Message::Success(UpdateInfo {
                update_available: true,
                latest_version: latest.to_string(),
                release_url: release.html_url,
            })
        } else {
            no_update()
        }
    }
}

async fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let release = client
        .get("https://api.github.com/repos/Fallstop/BluetoothKeySyncGUI/releases/latest")
        .header("User-Agent", "BluetoothKeySyncGUI-UpdateChecker")
        .send()
        .await?
        .json::<GitHubRelease>()
        .await?;

    Ok(release)
}
