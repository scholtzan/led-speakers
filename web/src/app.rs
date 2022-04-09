use crate::types::{Color, Error, Status, Theme, Themes, Visualization, Visualizations};

use inflector::Inflector;

use std::collections::VecDeque;
use yew::format::Json;
use yew::prelude::*;

use yew::services::fetch::FetchTask;
use yew::{html, Component, Html};

use std::collections::HashMap;

use crate::api;

/// Shared application state.
pub struct State {
    /// The identifier of the currently active visualization.
    current_visualization: String,

    /// Available visualizations.
    visualizations: Vec<Visualization>,

    /// The identifier of the currently active theme.
    current_theme: String,

    /// Available themes.
    themes: Vec<Theme>,

    /// The current speaker status.
    status: Status,

    /// Advanced/transformer settings.
    advanced_settings: HashMap<String, String>,

    /// The current custom theme config, if custom theme is currently selected.
    custom_theme: Option<Theme>,
}

/// Represents the application.
pub struct App {
    /// Shared application state.
    state: State,

    /// Active requests to the server.
    tasks: VecDeque<FetchTask>,

    /// Link to components for creating callbacks.    
    link: ComponentLink<Self>,
}

/// Messages sent through interaction with the frontend.
pub enum Message {
    /// Request available visualizations.
    GetVisualizations,

    /// Request available themes.
    GetThemes,

    /// Request the current speaker status.
    GetStatus,

    /// Turn speaker on.
    TurnOn,

    /// Turn speaker off.
    TurnOff,

    /// The speaker has been turned on.
    TurnOnSuccess,

    /// The speaker has been turned off.
    TurnOffSuccess,

    /// Available visualizations successfully retrieved from the server.
    GetVisualizationsSuccess(Visualizations),

    /// Themes successfully retrieved from the server.
    GetThemesSuccess(Themes),

    /// Speaker status successfully retrieved from the server.
    GetStatusSuccess(Status),

    /// Request to change the currently active visualization.
    ChangeVisualization(String),

    /// Request to change the currently active theme.
    ChangeTheme(String),

    /// Currently active visualization got changed.
    ChangeVisualizationSuccess(String),

    /// Currently active theme got changed.
    ChangeThemeSuccess(String),

    /// Request advanced/transformer settings.
    GetAdvancedSettings,

    /// Change advanced/transformer settings.
    ChangeAdvancedSetting(String, String),

    /// Advanced/transformer settings retrieved from server.
    GetAdvancedSettingsSuccess(HashMap<String, String>),

    /// Advanced/transformer settings successfully changed.
    ChangeAdvancedSettingsSuccess,

    /// Change visualization setting.
    ChangeVizSetting(String, String),

    /// Visualization settings got changed successfully.
    ChangeVizSettingSuccess,

    /// Add a new color to the custom theme.
    AddCustomThemeColor,

    /// Remove a color from the current custom theme.
    RemoveCustomThemeColor,

    /// Change a custom theme color.
    ChangeCustomThemeColor(usize, String),

    /// Request to update custom theme.
    UpdateCustomTheme,

    /// Custom theme update was successful.
    ChangeCustomThemeColorSuccess,

    /// Any error.
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
                custom_theme: None,
            },
            tasks: VecDeque::with_capacity(10),
            link,
        };

        // get information from server
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
                // request available visualizations
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
                // update the current and available viz
                self.state.visualizations = viz_info.visualizations;
                self.state.current_visualization = viz_info.current;
                true
            }
            Message::GetThemes => {
                // get available and the current theme
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
                // get the speaker status
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
                // update available and the current theme
                self.state.themes = theme_info.themes;
                self.state.current_theme = theme_info.current;
                true
            }
            Message::GetStatusSuccess(status) => {
                // update the status
                self.state.status = status;
                true
            }
            Message::ChangeVisualization(new_viz) => {
                // set a new active visualization
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
                // set a new active theme
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
                // update the viz
                self.state.current_visualization = new_viz;
                true
            }
            Message::ChangeThemeSuccess(new_theme) => {
                // update the theme
                self.state.current_theme = new_theme;
                self.state.custom_theme = None;
                true
            }
            Message::TurnOff => {
                // request to turn off the speaker
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
                // turn off
                self.state.status.is_stopped = true;
                true
            }
            Message::TurnOn => {
                // request to turn on the speaker
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
                // turn on
                self.state.status.is_stopped = false;
                true
            }
            Message::GetAdvancedSettings => {
                // request advanced/transformer settings
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
                // store settings
                self.state.advanced_settings = settings;
                true
            }
            Message::ChangeAdvancedSetting(key, value) => {
                // request to change advanced/transformer settings
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
                // request and change current viz settings
                let current_viz = self.state.current_visualization.clone();
                let current_viz = self
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
            Message::UpdateCustomTheme => {
                // request and change current theme
                if let Some(custom_theme) = self.state.custom_theme.as_mut() {
                    let new_theme = custom_theme.clone();
                    let handler = self
                        .link
                        .callback(move |response: api::FetchResponse<bool>| {
                            let (meta, Json(data)) = response.into_parts();
                            match data {
                                Ok(_) => Message::ChangeCustomThemeColorSuccess,
                                Err(err) => Message::Error(Error::FetchError(
                                    format!(
                                        "Error changing cusotm theme color: {:?}",
                                        err.to_string()
                                    ),
                                    meta,
                                )),
                            }
                        });

                    self.queue_task(api::set_custom_theme(new_theme, handler));
                }
                true
            }
            Message::ChangeCustomThemeColor(color_index, hex_color) => {
                // change a specific color of the current custom theme
                if let Some(custom_theme) = self.state.custom_theme.as_mut() {
                    custom_theme.colors[color_index] = Color::from_hex(&hex_color);
                    self.link.send_message(Message::UpdateCustomTheme);
                }
                true
            }
            Message::ChangeCustomThemeColorSuccess => true,
            Message::AddCustomThemeColor => {
                // add a new color to the custom theme; black by default
                if let Some(custom_theme) = self.state.custom_theme.as_mut() {
                    custom_theme.colors.push(Color::default())
                } else {
                    // no custom theme before, create new one
                    self.state.custom_theme = Some(Theme {
                        name: "custom".to_string(),
                        colors: vec![Color::default()],
                    })
                }
                self.link.send_message(Message::UpdateCustomTheme);
                true
            }
            Message::RemoveCustomThemeColor => {
                // remove a specific color from the custom theme
                if let Some(custom_theme) = self.state.custom_theme.as_mut() {
                    if custom_theme.colors.len() > 1 {
                        custom_theme.colors.pop();
                        self.link.send_message(Message::UpdateCustomTheme);
                    }
                }
                true
            }
            Message::Error(err) => {
                // log errors
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
        // callbacks
        let on_viz_change = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(e) = e {
                Message::ChangeVisualization(e.value())
            } else {
                Message::Error(Error::Misc("Cannot change visualization".to_string()))
            }
        });
        let on_theme_change = self.link.callback(|e: ChangeData| {
            if let ChangeData::Select(e) = e {
                if "custom".eq(&e.value()) {
                    Message::AddCustomThemeColor
                } else {
                    Message::ChangeTheme(e.value())
                }
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
        let change_custom_theme_color = |color_index: usize| {
            self.link.callback(move |e: ChangeData| {
                if let ChangeData::Value(val) = e {
                    Message::ChangeCustomThemeColor(color_index, val)
                } else {
                    Message::Error(Error::Misc("Cannot change custom theme".to_string()))
                }
            })
        };
        let add_custom_theme_color = self.link.callback(|_| Message::AddCustomThemeColor);
        let remove_custom_theme_color = self.link.callback(|_| Message::RemoveCustomThemeColor);

        // render page
        html! {
            <>
            <nav class="navbar is-dark" role="navigation" aria-label="main navigation">
                <div class="container">
                    <div class="navbar-brand">
                        <div class="navbar-item logo">
                        {"LED Speakers"}
                            <div class="field is-grouped on-off-button">
                                <p class="control">
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
                                </p>
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
                                <option value={"custom"}>{"Custom"}</option>
                                </select>
                            </div>
                        </div>
                        {
                            if self.state.custom_theme.is_none() {
                                html! {
                                    <div class="color-container">
                                    {
                                        for current_colors.iter().map(|color| self.view_color(&color))
                                    }
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="color-container">
                                    {
                                        for self.state.custom_theme.as_ref().unwrap().colors.iter().enumerate().map(|(i, color)| html! {
                                            <input type="color" value={color.to_hex()} onchange=change_custom_theme_color(i) />
                                        })
                                    }
                                    <button class="button is-rounded is-small color-control" onclick=add_custom_theme_color>{"+"}</button>
                                    <button class="button is-rounded is-small color-control" onclick=remove_custom_theme_color>{"–"}</button>
                                    </div>
                                }
                            }
                        }
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
    /// Renders the options for the drop down selects (themes, visualizations).
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

    /// Renders a color of the current theme as a circle.
    fn view_color(&self, color: &Color) -> Html {
        html! {
            <span class="color" style=format!("background-color: rgb({:?}, {:?}, {:?})", color.r, color.g, color.b)></span>
        }
    }

    /// Queues multiple requests to the server.
    fn queue_task(&mut self, task: FetchTask) {
        if self.tasks.capacity() <= 0 {
            self.tasks.pop_front();
        }

        self.tasks.push_back(task);
    }
}
