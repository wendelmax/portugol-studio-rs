use serde::{Deserialize, Serialize};

const ALERTS: &str = "https://api.github.com/repos/wendelmax/ferro-studio/issues?state=open&labels=alerta&per_page=20";

#[derive(Deserialize)]
struct Issue {
    title: String,
    body: Option<String>,
    pull_request: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub(crate) struct Alert {
    title: String,
    message: String,
}

fn parse_alerts(reader: impl std::io::Read) -> Result<Vec<Alert>, String> {
    let issues: Vec<Issue> = serde_json::from_reader(reader.take(512 * 1024))
        .map_err(|_| "O GitHub retornou avisos em um formato inesperado.")?;
    Ok(issues
        .into_iter()
        .filter(|issue| issue.pull_request.is_none())
        .map(|issue| Alert {
            title: issue.title,
            message: issue.body.unwrap_or_default(),
        })
        .collect())
}

fn fetch_alerts() -> Result<Vec<Alert>, String> {
    let response = crate::updates::github_get(ALERTS)
        .map_err(|_| "Não foi possível consultar os avisos do projeto no GitHub.")?;
    parse_alerts(response.into_reader())
}

#[tauri::command]
pub(crate) async fn project_alerts() -> Result<Vec<Alert>, String> {
    tauri::async_runtime::spawn_blocking(fetch_alerts)
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_public_alerts_and_ignores_pull_requests() {
        let response = br#"[{"title":"Aviso","body":"Mensagem"},{"title":"PR","body":null,"pull_request":{}}]"#;
        let alerts = parse_alerts(&response[..]).unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "Aviso");
        assert_eq!(alerts[0].message, "Mensagem");
    }
}
