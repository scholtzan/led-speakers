use actix_cors::Cors;
use actix_web::{get, http, post, web, App, Error, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use dyn_clone::DynClone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

mod app;
mod audio;
mod buffer;
mod led;
mod routes;
mod settings;
mod theme;
mod transform;
mod viz;

use crate::app::{AppState, Visualization};
use crate::audio::AudioStream;
use crate::routes::init;
use crate::settings::Settings;
use crate::transform::AudioTransformer;
use crate::viz::RotatingViz;
use crate::viz::VizRunner;

#[macro_use]
extern crate dotenv_codegen;

const ASSETS_DIR: &str = "web/static";
const CONFIG: &str = "config.json";

/// Serves the index.html file
async fn serve_index_file() -> Result<actix_files::NamedFile, Error> {
    Ok(
        actix_files::NamedFile::open(format!("{}/index.html", ASSETS_DIR))?
            .set_status_code(http::StatusCode::OK),
    )
}

#[actix_rt::main]
async fn main() -> Result<()> {
    // read settings from config.json
    let mut conf = config::Config::default();
    conf.merge(config::File::with_name(CONFIG)).unwrap();
    let settings: Settings = conf.try_into().unwrap();
    let visualizations = settings
        .visualizations
        .iter()
        .map(|v| Visualization {
            pretty_name: v.get_pretty_name().to_string(),
            identifier: v.get_name().to_string(),
            settings: None,
        })
        .collect::<Vec<Visualization>>()
        .clone();

    // new audio transformer instance from settings
    // has access to audio stream
    let mut transformer = AudioTransformer::new(
        settings.sink,
        settings.fft_len,
        settings.total_bands,
        settings.lower_cutoff,
        settings.upper_cutoff,
        settings.monstercat,
        settings.decay,
        settings.buffer_size,
    );
    transformer.start();

    // instantiate visualization
    let mut viz_left = dyn_clone::clone_box(
        &*settings
            .visualizations
            .into_iter()
            .find(|v| v.get_name() == "sparkle_viz")
            .unwrap(),
    );
    viz_left.set_total_pixels(settings.output.left.total_leds as usize);
    let mut viz_right = dyn_clone::clone_box(&*viz_left);

    // viz runner will update the visualization periodically
    let viz_runner = VizRunner {
        viz_left: Arc::new(Mutex::new(viz_left)),
        viz_right: Arc::new(Mutex::new(viz_right)),
        output_settings: settings.output.clone(),
        is_stopped: Arc::new(AtomicBool::from(false)),
        theme: settings.themes[0].clone(),
        transformer: Arc::new(Mutex::new(transformer)),
    };
    viz_runner.start();

    eprintln!("Start server");

    let host = settings.server_host.clone();
    let port = settings.server_port.clone();

    let shared_viz_runner = Arc::new(Mutex::new(viz_runner)).clone();
    let themes = settings.themes.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN)
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .allowed_header(http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                    .allowed_header(http::header::ALLOW)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(web::Data::new(AppState {
                viz_runner: shared_viz_runner.clone(),
                themes: themes.clone(),
                visualizations: visualizations.clone(),
            }))
            .configure(init)
            .service(
                actix_files::Files::new("/", ASSETS_DIR)
                    .index_file("index.html")
                    .default_handler(web::to(|| serve_index_file())),
            )
    })
    .bind(&format!("{}:{}", host, port))?
    .workers(1)
    .run()
    .await?;

    Ok(())
}
