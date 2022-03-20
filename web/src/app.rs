use crate::types::{Error, Theme, Themes, Visualization, Visualizations};

use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::StatusCode;
use yew::services::fetch::{FetchService, FetchTask};
use yew::{html, Component, Html, Properties};

use crate::api;

pub struct State {
    current_visualization: String,
    visualizations: Vec<Visualization>,

    current_theme: String,
    themes: Vec<Theme>,
}

pub struct App {
    state: State,
    task: Option<FetchTask>,
    link: ComponentLink<Self>,
}

pub enum Message {
    GetVisualizations,
    GetThemes,
    GetVisualizationsSuccess(Visualizations),
    GetThemesSuccess(Themes),
    Error(Error),
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let app = Self {
            state: State {
                current_visualization: "".to_string(),
                visualizations: Vec::new(),
                current_theme: "".to_string(),
                themes: Vec::new(),
            },
            task: None,
            link,
        };

        app.link.send_message(Message::GetVisualizations);
        app.link.send_message(Message::GetThemes);

        app
    }

    fn update(&mut self, message: Self::Message) -> ShouldRender {
        match message {
            Message::GetVisualizations => {
                let handler =
                    self.link
                        .callback(move |response: api::FetchResponse<Visualizations>| {
                            let (meta, Json(data)) = response.into_parts();
                            match data {
                                Ok(viz_info) => Message::GetVisualizationsSuccess(viz_info),
                                Err(err) => Message::Error(Error::FetchError(
                                    format!("Error getting visualizations: {:?}", err.to_string()),
                                    meta,
                                )),
                            }
                        });

                self.task = Some(api::get_visualizations(handler));
                true
            }
            Message::GetVisualizationsSuccess(viz_info) => {
                self.state.visualizations = viz_info.visualizations;
                self.state.current_visualization = viz_info.current;
                true
            }
            Message::GetThemes => true,
            Message::GetThemesSuccess(theme_info) => true,
            Message::Error(err) => true,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            // impl
        }
    }
}
