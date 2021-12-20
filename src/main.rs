use actix_cors::Cors;
use actix_web::{get, post, http, web, App, Error, HttpServer, HttpResponse, Responder};
use anyhow::Result;
use dotenv::dotenv;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use dyn_clone::DynClone;

mod audio;
mod buffer;
mod settings;
mod theme;
mod transform;
mod viz;

use crate::settings::Settings;
use crate::transform::AudioTransformer;
use crate::audio::AudioStream;
use crate::viz::VizRunner;
use crate::viz::RotatingViz;

#[macro_use]
extern crate dotenv_codegen;


const ASSETS_DIR: &str = "../web/dist/spa"; // todo
const CONFIG: &str = "config.json";

async fn serve_index_file() -> Result<actix_files::NamedFile, Error> {
    Ok(
        actix_files::NamedFile::open(format!("{}/index.html", ASSETS_DIR))?
            .set_status_code(http::StatusCode::OK),
    )
}


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_viz);
    cfg.service(get_themes);
    cfg.service(get_viz_types);
    cfg.service(set_viz);
}


#[get("/api/vizualization")]
async fn get_viz() -> impl Responder {
    // HttpResponse::Ok().json(tags)
    HttpResponse::Ok().json("Todo: get viz")
}

#[get("/api/theme")]
async fn get_themes() -> impl Responder {
    // HttpResponse::Ok().json(tags)
    HttpResponse::Ok().json("Todo: get themes")
}

#[get("/api/type")]
async fn get_viz_types() -> impl Responder {
    // HttpResponse::Ok().json(tags)
    HttpResponse::Ok().json("Todo: get viz types")
}

#[post("/api/viz")]
async fn set_viz(
    // viz: web::Json<VizualizationRequest>,
) -> impl Responder {
    HttpResponse::Ok().json("Todo: update viz")
}

struct AppState {
    vizualization: Mutex<VizRunner>
}


#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();

    eprintln!("Start LED speakers");

    let mut conf = config::Config::default();
    conf.merge(config::File::with_name(CONFIG)).unwrap();
    let settings: Settings = conf.try_into().unwrap();

    let mut transformer = AudioTransformer::new(
        settings.sink, 
        settings.bins, 
        settings.total_bands, 
        settings.lower_cutoff, 
        settings.upper_cutoff, 
        settings.monstercat, 
        settings.decay
    );
    transformer.start();

    let viz = dyn_clone::clone_box(&*settings.vizualizations.into_iter().find(|v| v.get_name() == "rotating_viz").unwrap());
    let app_state = web::Data::new(AppState {
        vizualization: Mutex::new(VizRunner {
            viz: Arc::new(Mutex::new(viz)),
            is_stopped: Arc::new(AtomicBool::from(true))
        })
    });

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
                    .allowed_header(http::header::ALLOW),
            )
            .configure(init)
            .service(
                actix_files::Files::new("/", ASSETS_DIR)
                    .index_file("index.html")
                    .default_handler(web::to(|| serve_index_file())),
            )
    })
    .bind(&format!(
        "{}:{}",
        dotenv!("SERVER_HOST"),
        dotenv!("SERVER_PORT")
    ))?
    .workers(1)
    .run()
    .await?;

    Ok(())
}
