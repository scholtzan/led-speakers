use led_speakers::theme::{Color, Theme};

#[test]
fn theme_from_json() {
    let theme_json = r#"
        {
            "name": "test",
            "colors": [
                [0, 0, 255, 1],
                [1, 1, 1, 1]
            ]
        }
    "#;
    let theme: Theme = serde_json::from_str(theme_json).unwrap();

    assert_eq!(theme.name, "test");
    assert_eq!(theme.colors[0], Color {r: 0, g: 0, b: 255, a: 1});
    assert_eq!(theme.colors[1], Color{r: 1, g: 1, b: 1, a: 1});
}

#[test]
fn color_from_vec() {
    let color = Color::from_vec(&vec![1, 2, 3, 4]);
    assert_eq!(color.r, 1);
    assert_eq!(color.g, 2);
    assert_eq!(color.b, 3);
    assert_eq!(color.a, 4);
}
