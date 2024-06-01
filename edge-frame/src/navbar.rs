const CUSTOM_STYLES: &str = r#"
    .navbar-height {
        --bulma-navbar-height: 1rem;
    }

    .navbar-item, .navbar-link, .navbar-dropdown {
        padding-left: 1.5rem;
        padding-right: 1.5rem;
        padding-top: 0.1rem;
        padding-bottom: 0.1rem;
        border-radius: 3px;
        align-self: center;
    }

    .navbar-item > .buttons > .button {
        padding-top: 0.1rem;
        padding-bottom: 0.1rem;
        font-weight: normal;
    }

    .navbar-dropdown {
        padding-left: 0rem;
        padding-right: 0rem;
    }

    .navbar-dropdown > .navbar-item {
        padding-top: 0.3rem;
        padding-bottom: 0.3rem;
        padding-inline-end: 6rem;
    }

    .navbar-dropdown > a.navbar-item {
        padding-top: 0.3rem;
        padding-bottom: 0.3rem;
        padding-inline-end: 6rem;
    }
"#;

pub fn inject_custom_styles() -> Option<()> {
    let document = web_sys::window()?.document()?;

    let head = document.get_elements_by_tag_name("head").item(0)?;
    let style = document.create_element("style").ok()?;

    style.set_attribute("type", "text/css").ok()?;
    style
        .append_child(&document.create_text_node(CUSTOM_STYLES))
        .ok()?;

    head.append_child(&style).ok()?;

    Some(())
}
