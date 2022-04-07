use actix_web::{get, post, put, web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::Arc;

use crate::app::{AppState, Visualization};
use crate::settings::TransformerSettings;
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

#[derive(Serialize, Deserialize, Clone)]
struct ChangeVisualization {
    pub visualization: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChangeTheme {
    pub theme: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct StatusResponse {
    pub is_stopped: bool,
}

/// Initializes available routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_viz);
    cfg.service(get_themes);
    cfg.service(update_visualization);
    cfg.service(update_theme);
    cfg.service(turn_off);
    cfg.service(turn_on);
    cfg.service(get_status);
    cfg.service(get_transformer_settings);
    cfg.service(update_transformer_settings);
    cfg.service(update_viz_settings);
    cfg.service(set_custom_theme);
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
        current: current_theme.theme.lock().unwrap().name.clone(),
        themes: themes,
    };
    HttpResponse::Ok().json(response)
}

#[put("/api/visualization")]
async fn update_visualization(
    new_visualization: web::Json<ChangeVisualization>,
    data: web::Data<AppState>,
) -> impl Responder {
    if let Some(viz) = data
        .settings
        .lock()
        .unwrap()
        .visualizations
        .iter()
        .find(|&v| v.get_name() == new_visualization.visualization)
    {
        data.viz_runner
            .lock()
            .unwrap()
            .set_visualization(dyn_clone::clone_box(&**viz));
        HttpResponse::Ok().json(true)
    } else {
        HttpResponse::Ok().json(false)
    }
}

#[put("/api/theme")]
async fn update_theme(
    new_theme: web::Json<ChangeTheme>,
    data: web::Data<AppState>,
) -> impl Responder {
    let themes = data.themes.clone();
    let new_theme = themes.into_iter().find(|t| t.name == new_theme.theme);
    if let Some(theme) = new_theme {
        data.viz_runner.lock().unwrap().set_theme(theme);
        HttpResponse::Ok().json(true)
    } else {
        HttpResponse::Ok().json(false)
    }
}

#[post("/api/theme/custom")]
async fn set_custom_theme(theme: web::Json<Theme>, data: web::Data<AppState>) -> impl Responder {
    data.viz_runner
        .lock()
        .unwrap()
        .set_theme(theme.into_inner());
    HttpResponse::Ok().json(true)
}

#[post("/api/on")]
async fn turn_on(data: web::Data<AppState>) -> impl Responder {
    data.viz_runner.lock().unwrap().stop(false);
    HttpResponse::Ok().json(true)
}

#[post("/api/off")]
async fn turn_off(data: web::Data<AppState>) -> impl Responder {
    data.viz_runner.lock().unwrap().stop(true);
    HttpResponse::Ok().json(true)
}

#[get("/api/status")]
async fn get_status(data: web::Data<AppState>) -> impl Responder {
    // Return the current and available visualizations.
    let current_viz = data.viz_runner.lock().unwrap();
    let response = StatusResponse {
        is_stopped: current_viz.is_stopped(),
    };
    HttpResponse::Ok().json(response)
}

#[get("/api/settings")]
async fn get_transformer_settings(data: web::Data<AppState>) -> impl Responder {
    // Return the current settings.
    let settings = data.settings.lock().unwrap().transformer.clone();
    HttpResponse::Ok().json(settings.to_map())
}

#[put("/api/settings")]
async fn update_transformer_settings(
    new_settings: web::Json<HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let transformer_settings = TransformerSettings::from_map(new_settings.into_inner());
    data.settings
        .lock()
        .unwrap()
        .apply_transformer_settings(transformer_settings.clone());
    data.viz_runner
        .lock()
        .unwrap()
        .update_transformer_settings(transformer_settings);
    HttpResponse::Ok().json(true)
}

#[put("/api/visualization/{id}")]
async fn update_viz_settings(
    new_settings: web::Json<HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    data.viz_runner
        .lock()
        .unwrap()
        .update_viz_settings(new_settings.into_inner());
    HttpResponse::Ok().json(true)
}
