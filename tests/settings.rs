use led_speakers::settings::Settings;
use led_speakers::theme::{Color, Theme};

#[test]
fn settings_from_json() {
    let settings_json = r#"
        {
            "vizualizations": {
                "rotating_viz": {
                    "pretty_name": "Rotating Viz"
                }
            },
            "themes": [
                {
                    "name": "test",
                    "colors": [
                        [0, 0, 255, 1],
                        [1, 1, 1, 1]
                    ]
                }
            ]
        }
    "#;
    let settings: Settings = serde_json::from_str(settings_json).unwrap();

    assert_eq!(
        settings.themes[0],
        Theme {
            name: "test".to_string(),
            colors: vec![
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 1
                },
                Color {
                    r: 1,
                    g: 1,
                    b: 1,
                    a: 1
                }
            ]
        }
    );
    assert_eq!(
        settings.vizualizations[0].as_ref().get_pretty_name(),
        "Rotating Viz".to_string()
    );
    assert_eq!(
        settings.vizualizations[0].as_ref().get_name(),
        "rotating_viz".to_string()
    );
}
