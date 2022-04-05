use crate::types::{Color, Error, Status, Theme, Themes, Visualization, Visualizations};

use inflector::Inflector;
use log::info;
use std::collections::VecDeque;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::StatusCode;
use yew::services::fetch::{FetchService, FetchTask};
use yew::{html, Component, Html, Properties};

use std::collections::HashMap;

use crate::api;

pub struct State {
    current_visualization: String,
    visualizations: Vec<Visualization>,

    current_theme: String,
    themes: Vec<Theme>,

    status: Status,
    advanced_settings: HashMap<String, String>,
}

pub struct App {
    state: State,
    tasks: VecDeque<FetchTask>,
    link: ComponentLink<Self>,
}

pub enum Message {
    GetVisualizations,
    GetThemes,
    GetStatus,
    TurnOn,
    TurnOff,
    TurnOnSuccess,
    TurnOffSuccess,
    GetVisualizationsSuccess(Visualizations),
    GetThemesSuccess(Themes),
    GetStatusSuccess(Status),
    ChangeVisualization(String),
    ChangeTheme(String),
    ChangeVisualizationSuccess(String),
    ChangeThemeSuccess(String),
    GetAdvancedSettings,
    ChangeAdvancedSetting(String, String),
    GetAdvancedSettingsSuccess(HashMap<String, String>),
    ChangeAdvancedSettingsSuccess,
    ChangeVizSetting(String, String),
    ChangeVizSettingSuccess,
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
                status: Status { is_stopped: false },
                advanced_settings: HashMap::new(),
            },
            tasks: VecDeque::with_capacity(10),
            link,
        };

        app.link.send_message_batch(vec![
            Message::GetVisualizations,
            Message::GetThemes,
            Message::GetStatus,
            Message::GetAdvancedSettings,
        ]);
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

                self.queue_task(api::get_visualizations(handler));
                true
            }
            Message::GetVisualizationsSuccess(viz_info) => {
                self.state.visualizations = viz_info.visualizations;
                self.state.current_visualization = viz_info.current;
                true
            }
            Message::GetThemes => {
                let handler = self
                    .link
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

                self.queue_task(api::get_themes(handler));
                true
            }
            Message::GetStatus => {
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<Status>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(viz_info) => Message::GetStatusSuccess(viz_info),
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error getting status: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::get_status(handler));
                true
            }
            Message::GetThemesSuccess(theme_info) => {
                self.state.themes = theme_info.themes;
                self.state.current_theme = theme_info.current;
                true
            }
            Message::GetStatusSuccess(status) => {
                self.state.status = status;
                true
            }
            Message::ChangeVisualization(new_viz) => {
                let new_viz_success = new_viz.clone();
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::ChangeVisualizationSuccess(new_viz_success.clone()),
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error changing visualization: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::update_visualization(new_viz, handler));
                true
            }
            Message::ChangeTheme(new_theme) => {
                let new_theme_success = new_theme.clone();
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::ChangeThemeSuccess(new_theme_success.clone()),
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error changing theme: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::update_theme(new_theme, handler));
                true
            }
            Message::ChangeVisualizationSuccess(new_viz) => {
                self.state.current_visualization = new_viz;
                false
            }
            Message::ChangeThemeSuccess(new_theme) => {
                self.state.current_theme = new_theme;
                true
            }
            Message::TurnOff => {
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::TurnOffSuccess,
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error turning off: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::turn_off(handler));
                true
            }
            Message::TurnOffSuccess => {
                self.state.status.is_stopped = true;
                true
            }
            Message::TurnOn => {
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::TurnOnSuccess,
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error turning on: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::turn_on(handler));
                true
            }
            Message::TurnOnSuccess => {
                self.state.status.is_stopped = false;
                true
            }
            Message::GetAdvancedSettings => {
                log::info!("get: ");
                let handler = self.link.callback(
                    move |response: api::FetchResponse<HashMap<String, String>>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(settings) => Message::GetAdvancedSettingsSuccess(settings),
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error getting advanced settings: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    },
                );

                self.queue_task(api::get_advanced_settings(handler));
                true
            }
            Message::GetAdvancedSettingsSuccess(settings) => {
                self.state.advanced_settings = settings;

                log::info!("Update: {:?}", self.state.advanced_settings);
                true
            }
            Message::ChangeAdvancedSetting(key, value) => {
                self.state.advanced_settings.insert(key, value);
                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::ChangeAdvancedSettingsSuccess,
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error changing advanced settings: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });

                self.queue_task(api::update_advanced_settings(
                    self.state.advanced_settings.clone(),
                    handler,
                ));
                false
            }
            Message::ChangeAdvancedSettingsSuccess => true,
            Message::ChangeVizSetting(key, value) => {
                let current_viz = self.state.current_visualization.clone();
                let mut current_viz = self
                    .state
                    .visualizations
                    .iter_mut()
                    .find(|viz| viz.identifier == current_viz)
                    .unwrap();
                current_viz.settings.as_mut().unwrap().insert(key, value);

                let handler = self
                    .link
                    .callback(move |response: api::FetchResponse<bool>| {
                        let (meta, Json(data)) = response.into_parts();
                        match data {
                            Ok(_) => Message::ChangeVizSettingSuccess,
                            Err(err) => Message::Error(Error::FetchError(
                                format!("Error changing viz settings: {:?}", err.to_string()),
                                meta,
                            )),
                        }
                    });
                let updated_viz = current_viz.clone();

                self.queue_task(api::update_visualization_settings(updated_viz, handler));
                false
            }
            Message::ChangeVizSettingSuccess => true,
            Message::Error(err) => {
                if let Error::FetchError(msg, _) = err {
                    log::info!("Error {:?}", msg);
                } else if let Error::Misc(msg) = err {
                    log::info!("Error {:?}", msg);
                }
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let on_viz_change = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(e) = e {
                Message::ChangeVisualization(e.value())
            } else {
                Message::Error(Error::Misc("Cannot change visualization".to_string()))
            }
        });
        let on_theme_change = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(e) = e {
                Message::ChangeTheme(e.value())
            } else {
                Message::Error(Error::Misc("Cannot change theme".to_string()))
            }
        });
        let current_theme = self
            .state
            .themes
            .iter()
            .find(|theme| theme.name == self.state.current_theme);
        let current_colors = match current_theme {
            Some(theme) => theme.colors.clone(),
            _ => Vec::new(),
        };
        let status = self.state.status.clone();
        let turn_on = self.link.callback(|_| Message::TurnOn);
        let turn_off = self.link.callback(|_| Message::TurnOff);
        let on_advanced_setting_changed = |setting: String| {
            let setting = setting.clone();
            self.link.callback(move |e: ChangeData| {
                if let ChangeData::Value(val) = e {
                    Message::ChangeAdvancedSetting(setting.clone(), val)
                } else {
                    Message::Error(Error::Misc("Cannot change settings".to_string()))
                }
            })
        };
        let on_viz_setting_changed = |setting: String| {
            let setting = setting.clone();
            self.link.callback(move |e: ChangeData| {
                if let ChangeData::Value(val) = e {
                    Message::ChangeVizSetting(setting.clone(), val)
                } else {
                    Message::Error(Error::Misc("Cannot change viz settings".to_string()))
                }
            })
        };

        html! {
            <>
            <nav class="navbar is-dark" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <div class="navbar-item">
                    {"LED Speakers"}
                    </div>
                </div>

                <div class="navbar-menu">
                    <div class="navbar-end">
                        <div class="navbar-item">
                            <div class="buttons">
                            {
                                if status.is_stopped {
                                    html! {
                                        <a class="button is-black" onclick=turn_on>
                                        <strong>{"Turn on"}</strong>
                                        </a>
                                    }
                                } else {
                                    html! {
                                        <a class="button is-primary" onclick=turn_off>
                                        <strong>{"Turn off"}</strong>
                                        </a>
                                    }
                                }
                            }
                            </div>
                        </div>
                    </div>
                </div>
            </nav>

            <div class="container">
                <section class="section">
                    <h2 class="subtitle">{"Visualization"}</h2>
                    <div class="field">
                        <div class="control">
                            <div class="select">
                                <select onchange=on_viz_change>
                                {
                                    for self.state.visualizations.iter()
                                    .map(|viz| self.view_select_option(&viz.pretty_name, &viz.identifier, viz.identifier == self.state.current_visualization))
                                }
                                </select>
                            </div>
                        </div>
                    </div>

                    <details>
                        <summary>
                            {"Settings"}
                        </summary>
                        <p>
                        {
                            for self.state.visualizations.iter().map(|v| {
                                if v.identifier == self.state.current_visualization {
                                    html! {
                                        for v.settings.clone().unwrap().iter().map(|(key, val)| {
                                            html! {
                                                <div class="field">
                                                    <label class="label">{key.replace("_", ".").to_title_case()}</label>
                                                    <input class="input" type="text" value={val.to_string()} onchange=on_viz_setting_changed(key.to_string()) />
                                                </div>
                                            }
                                        })
                                    }
                                } else {
                                    html! {}
                                }
                            })
                        }
                        </p>
                    </details>
                </section>
                <section class="section">
                    <h2 class="subtitle">{"Theme"}</h2>

                    <div class="field">
                        <div class="control">
                            <div class="select">
                                <select onchange=on_theme_change>
                                {
                                    for self.state.themes.iter()
                                    .map(|theme| self.view_select_option(&theme.name, &theme.name, theme.name == self.state.current_theme))
                                }
                                </select>
                            </div>
                        </div>
                        <div class="color-container">
                        {
                            for current_colors.iter().map(|color| self.view_color(&color))
                        }
                        </div>
                    </div>
                </section>
                <section class="section">
                    <h2 class="subtitle">{"Advanced Settings"}</h2>
                    {
                        for self.state.advanced_settings.iter().map(|(key, val)| {
                            html! {
                                <div class="field">
                                    <label class="label">{key.replace("_", ".").to_title_case()}</label>
                                    <input class="input" type="text" value={val.to_string()} onchange=on_advanced_setting_changed(key.to_string()) />
                                </div>
                            }
                        })
                    }
                </section>
            </div>
            </>
        }
    }
}

impl App {
    fn view_select_option(&self, select_option: &str, option_value: &str, selected: bool) -> Html {
        if selected {
            html! {
                <option selected=true value=option_value.to_string()>{select_option}</option>
            }
        } else {
            html! {
                <option value=option_value.to_string()>{select_option}</option>
            }
        }
    }

    fn view_color(&self, color: &Color) -> Html {
        html! {
            <span style=format!("border-radius: 50%; margin-top: 10px; margin-right: 10px; display: inline-block; height: 25px; width: 25px; background-color: rgb({:?}, {:?}, {:?})", color.r, color.g, color.b)></span>
        }
    }

    fn queue_task(&mut self, task: FetchTask) {
        if self.tasks.capacity() <= 0 {
            self.tasks.pop_front();
        }

        self.tasks.push_back(task);
    }
}
