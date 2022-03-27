use crate::types::{ChangeTheme, ChangeVisualization, Themes, Visualizations};

use anyhow::Error;
use yew::callback::Callback;
use yew::format::{Json, Nothing};
use yew::services::fetch::{Credentials, FetchOptions, FetchService, FetchTask, Request, Response};

pub type FetchResponse<T> = Response<Json<Result<T, Error>>>;
type FetchCallback<T> = Callback<FetchResponse<T>>;
const SERVER_HOST: &str = dotenv!("SERVER_HOST");
const SERVER_PORT: &str = dotenv!("SERVER_PORT");

fn url(path: &str) -> String {
    format!("http://{}:{}{}", SERVER_HOST, SERVER_PORT, path)
}

pub fn get_visualizations(callback: FetchCallback<Visualizations>) -> FetchTask {
    let req = Request::get(url("/api/visualization"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

pub fn get_themes(callback: FetchCallback<Themes>) -> FetchTask {
    let req = Request::get(url("/api/theme"))
        .header("Content-Type", "application/json")
        .body(Nothing)
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}

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

pub fn update_theme(new_theme: String, callback: FetchCallback<bool>) -> FetchTask {
    let body = ChangeTheme { theme: new_theme };
    let req = Request::put(url("/api/theme"))
        .header("Content-Type", "application/json")
        .body(Json(&body))
        .unwrap();

    FetchService::fetch(req, callback).unwrap()
}
