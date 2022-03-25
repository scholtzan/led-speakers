use actix_web::{get, http, post, web, App, Error, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex, Weak};

use crate::app::{AppState, Visualization};
use crate::theme::Theme;

#[derive(Serialize, Deserialize, Clone)]
struct VisualizationsResponse {
    current: String,
    visualizations: Vec<Visualization>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ThemesResponse {
    current: String,
    themes: Vec<Theme>,
}

/// Initializes available routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_viz);
    cfg.service(get_themes);
    cfg.service(set_viz);
}

#[get("/api/visualization")]
async fn get_viz(data: web::Data<AppState>) -> impl Responder {
    // Return the current and available visualizations.
    let visualizations = data.visualizations.clone();
    let current_viz = data.viz_runner.lock().unwrap();
    let response = VisualizationsResponse {
        current: Arc::clone(&current_viz.viz_left)
            .lock()
            .unwrap()
            .get_name()
            .to_string(),
        visualizations: visualizations,
    };
    HttpResponse::Ok().json(response)
}

#[get("/api/theme")]
async fn get_themes(data: web::Data<AppState>) -> impl Responder {
    let themes = data.themes.clone();
    let current_theme = data.viz_runner.lock().unwrap();
    let response = ThemesResponse {
        current: current_theme.theme.name.clone(),
        themes: themes,
    };
    HttpResponse::Ok().json(response)
}

#[post("/api/visualization")]
async fn set_viz() -> impl Responder {
    HttpResponse::Ok().json("Todo: update viz")
}
