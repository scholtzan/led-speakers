use actix_web::{get, http, put, web, App, Error, HttpResponse, HttpServer, Responder};
use dyn_clone::DynClone;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex, Weak};

use crate::app::{AppState, Visualization};
use crate::theme::Theme;
use crate::viz::Viz;

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

/// Initializes available routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_viz);
    cfg.service(get_themes);
    cfg.service(update_visualization);
    cfg.service(update_theme);
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

#[put("/api/visualization")]
async fn update_visualization(
    new_visualization: web::Json<ChangeVisualization>,
    data: web::Data<AppState>,
) -> impl Responder {
    let new_viz: Option<&Box<dyn Viz>> = data
        .settings
        .visualizations
        .iter()
        .find(|&v| v.get_name() == new_visualization.visualization);

    if let Some(viz) = new_viz {
        data.viz_runner.lock().unwrap().viz_left =
            Arc::new(Mutex::new(dyn_clone::clone_box(&**viz)));
        data.viz_runner.lock().unwrap().viz_right =
            Arc::new(Mutex::new(dyn_clone::clone_box(&**viz)));
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
        data.viz_runner.lock().unwrap().theme = theme;
        HttpResponse::Ok().json(true)
    } else {
        HttpResponse::Ok().json(false)
    }
}
