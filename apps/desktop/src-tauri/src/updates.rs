use std::io::Read as _;
use std::time::Duration;

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

const LATEST: &str = "https://api.github.com/repos/wendelmax/ferro-studio/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    body: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum UpdateInfo {
    Available {
        current: String,
        latest: String,
        notes: String,
    },
    Current {
        current: String,
        latest: String,
    },
    Unpublished {
        current: String,
    },
}

fn compare(current: &Version, release: Release) -> Result<UpdateInfo, String> {
    let tag = release.tag_name.trim_start_matches('v');
    let latest =
        Version::parse(tag).map_err(|_| "A versão publicada tem um identificador inválido.")?;
    if latest.cmp_precedence(current).is_gt() {
        Ok(UpdateInfo::Available {
            current: current.to_string(),
            latest: latest.to_string(),
            notes: release.body.unwrap_or_default(),
        })
    } else {
        Ok(UpdateInfo::Current {
            current: current.to_string(),
            latest: latest.to_string(),
        })
    }
}

pub(crate) fn github_get(url: &str) -> Result<ureq::Response, Box<ureq::Error>> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .user_agent("Portugol-Studio-RS")
        .build();
    agent
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(Box::new)
}

fn latest_release(current: &Version) -> Result<UpdateInfo, String> {
    let response = match github_get(LATEST) {
        Ok(response) => response,
        Err(error) if matches!(*error, ureq::Error::Status(404, _)) => {
            return Ok(UpdateInfo::Unpublished {
                current: current.to_string(),
            });
        }
        Err(_) => return Err("Não foi possível consultar as versões publicadas no GitHub.".into()),
    };
    let release: Release = serde_json::from_reader(response.into_reader().take(512 * 1024))
        .map_err(|_| "O GitHub retornou informações de versão em um formato inesperado.")?;
    compare(current, release)
}

#[tauri::command]
pub(crate) async fn check_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.clone();
    tauri::async_runtime::spawn_blocking(move || latest_release(&current))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_prerelease_and_numeric_versions() {
        let release = || Release {
            tag_name: "v0.10.0".into(),
            body: Some("Notas".into()),
        };
        assert!(matches!(
            compare(&Version::new(0, 9, 0), release()),
            Ok(UpdateInfo::Available { .. })
        ));
        assert!(matches!(
            compare(&Version::parse("0.10.0-rc.1").unwrap(), release()),
            Ok(UpdateInfo::Available { .. })
        ));
        assert!(matches!(
            compare(&Version::new(0, 10, 0), release()),
            Ok(UpdateInfo::Current { .. })
        ));
    }
}
