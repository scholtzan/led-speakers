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
            Message::GetThemes => {
                let handler =
                    self.link
                        .callback(move |response: api::FetchResponse<Themes>| {
                            let (meta, Json(data)) = response.into_parts();
                            match data {
                                Ok(viz_info) => Message::GetThemesSuccess(viz_info),
                                Err(err) => Message::Error(Error::FetchError(
                                    format!("Error getting themes: {:?}", err.to_string()),
                                    meta,
                                )),
                            }
                        });

                self.task = Some(api::get_themes(handler));
                true
            },
            Message::GetThemesSuccess(theme_info) => {
                self.state.themes = theme_info.themes;
                self.state.current_theme = theme_info.current;
                true
            },
            Message::Error(err) => true,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
            <div class="container">
                <nav class="navbar" role="navigation" aria-label="main navigation">
                    <div class="navbar-brand">
                    {"LED Speakers"}

                    <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="navbarBasicExample">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                    </div>

                    <div class="navbar-menu">
                        <div class="navbar-end">
                            <div class="navbar-item">
                                <div class="buttons">
                                    <a class="button is-primary">
                                    <strong>{"Turn on"}</strong>
                                    </a>
                                </div>
                            </div>
                        </div>
                    </div>
                </nav>

                <div>
                    <div class="field">
                        <label class="label">{"Visualization"}</label>
                        <div class="control">
                            <div class="select">
                                <select>
                                {
                                    for self.state.visualizations.iter()
                                    .map(|viz| self.view_select_option(&viz.pretty_name, viz.identifier == self.state.current_visualization))
                                }
                                </select>
                            </div>
                        </div>
                    </div>

                    <div class="field">
                        <label class="label">{"Theme"}</label>
                        <div class="control">
                            <div class="select">
                                <select>
                                {
                                    for self.state.themes.iter()
                                    .map(|theme| self.view_select_option(&theme.name, theme.name == self.state.current_theme))
                                }
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            </>
        }
    }
}

impl App {
    fn view_select_option(&self, select_option: &str, selected: bool) -> Html {
        if selected {
            html! {
                <option selected=true>{select_option}</option>
            }
        } else {
            html! {
                <option>{select_option}</option>
            }
        }
    }
}
