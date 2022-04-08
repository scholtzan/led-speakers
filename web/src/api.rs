use crate::types::{
    ChangeTheme, ChangeVisualization, Status, Theme, Themes, Visualization, Visualizations,
};

use anyhow::Error;

use std::collections::HashMap;
use yew::callback::Callback;
use yew::format::{Json, Nothing};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

pub type FetchResponse<T> = Response<Json<Result<T, Error>>>;
type FetchCallback<T> = Callback<FetchResponse<T>>;
const SERVER_HOST: &str = dotenv!("SERVER_HOST");
const SERVER_PORT: &str = dotenv!("SERVER_PORT");

/// Creates the request URL to the server.
fn url(path: &str) -> String {
    format!("http://{}:{}{}", SERVER_HOST, SERVER_PORT, path)
}

/// Returns all available and the currently active visualization.
pub fn get_visualizations(callback: FetchCallback<Visualizations>) -> FetchTask {
    let req = Request::get(url("/api/visualization"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Returns all available and the currently active theme.
pub fn get_themes(callback: FetchCallback<Themes>) -> FetchTask {
    let req = Request::get(url("/api/theme"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Sets a new active visualization.
pub fn update_visualization(new_viz: String, callback: FetchCallback<bool>) -> FetchTask {
    let body = ChangeVisualization {
        visualization: new_viz,
    };
    let req = Request::put(url("/api/visualization"))
        .header("Content-Type", "application/json")
        .body(Json(&body))
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Sets a new active theme.
pub fn update_theme(new_theme: String, callback: FetchCallback<bool>) -> FetchTask {
    let body = ChangeTheme { theme: new_theme };
    let req = Request::put(url("/api/theme"))
        .header("Content-Type", "application/json")
        .body(Json(&body))
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Returns the status of the speakers.
/// Whether they are turned on or off.
pub fn get_status(callback: FetchCallback<Status>) -> FetchTask {
    let req = Request::get(url("/api/status"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Turns the speakers on.
pub fn turn_on(callback: FetchCallback<bool>) -> FetchTask {
    let req = Request::post(url("/api/on"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Turns the speakers off.
pub fn turn_off(callback: FetchCallback<bool>) -> FetchTask {
    let req = Request::post(url("/api/off"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Returns advanced/transformer settings.
pub fn get_advanced_settings(callback: FetchCallback<HashMap<String, String>>) -> FetchTask {
    let req = Request::get(url("/api/settings"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Changes the advanced/transformer settings to the new values provided.
pub fn update_advanced_settings(
    settings: HashMap<String, String>,
    callback: FetchCallback<bool>,
) -> FetchTask {
    let req = Request::put(url("/api/settings"))
        .header("Content-Type", "application/json")
        .body(Json(&settings))
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Changes the settings of the currently active visualization to the values provided.
pub fn update_visualization_settings(
    visualization: Visualization,
    callback: FetchCallback<bool>,
) -> FetchTask {
    let req = Request::put(url(&format!(
        "/api/visualization/{}",
        visualization.identifier
    )))
    .header("Content-Type", "application/json")
    .body(Json(&visualization.settings))
    .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

/// Sets a theme with custom colors that are provided in the body as the active theme.
pub fn set_custom_theme(theme: Theme, callback: FetchCallback<bool>) -> FetchTask {
    let req = Request::post(url("/api/theme/custom"))
        .header("Content-Type", "application/json")
        .body(Json(&theme))
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}
