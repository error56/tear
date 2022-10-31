extern crate lazy_static;

use std::{collections::HashMap, path::Component};

use async_minecraft_ping::{
    ConnectionConfig, ServerDescription, ServerDescriptionComponent, ServerError,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref COLOR_MAP: HashMap<&'static str, &'static str> = [
        ("black", "#000000"),
        ("dark_blue", "#0000aa"),
        ("dark_green", "#00aa00"),
        ("dark_aqua", "#00aaaa"),
        ("dark_red", "#aa0000"),
        ("dark_purple", "#aa00aa"),
        ("gold", "#ffaa00"),
        ("gray", "#aaaaaa"),
        ("dark_gray", "#555555"),
        ("blue", "#5555ff"),
        ("green", "#55ff55"),
        ("aqua", "#55ffff"),
        ("red", "#ff5555"),
        ("light_purple", "#ff55ff"),
        ("yellow", "#ffff55"),
        ("white", "#ffffff")
    ]
    .iter()
    .copied()
    .collect();
    static ref COLOR_INDEX: HashMap<char, &'static str> = [
        ('0', "black"),
        ('1', "dark_blue"),
        ('2', "dark_green"),
        ('3', "dark_aqua"),
        ('4', "dark_red"),
        ('5', "dark_purple"),
        ('6', "gold"),
        ('7', "gray"),
        ('8', "dark_gray"),
        ('9', "blue"),
        ('a', "green"),
        ('b', "aqua"),
        ('c', "red"),
        ('d', "light_purple"),
        ('e', "yellow"),
        ('f', "white")
    ]
    .iter()
    .copied()
    .collect();
}

#[allow(unused)]
#[derive(Debug)]
struct Server {
    address: String,
    port: Option<u16>,
    clean_motd: String,
    html_motd: String,
}

impl Server {
    pub fn process_motd(&mut self, components: &Vec<ServerDescriptionComponent>) {
        self.generate_html(components);
        self.generate_clean(components);
    }

    pub fn process_legacy_motd(&mut self, text: &str) {
        self.process_motd(&self.convert_old_format_to_components(text));
    }

    fn convert_old_format_to_components(&self, text: &str) -> Vec<ServerDescriptionComponent> {
        let mut components = vec![];

        let mut chars = text.chars();

        let mut bold = None;
        let mut italic = None;
        let mut strikethrough = None;
        let mut obfuscated = None;
        let mut underlined = None;
        let mut color = Some("white".to_owned());
        let mut style_change = false;

        let mut txt = String::new();

        loop {
            let c = chars.nth(0);

            if c.is_none() {
                // lets end previous component handling
                let mut component = ServerDescriptionComponent::default();
                component.text = txt;
                component.bold = bold;
                component.italic = italic;
                component.strikethrough = strikethrough;
                component.obfuscated = obfuscated;
                component.underlined = underlined;
                component.color = color;

                // push constructed component to vector
                components.push(component);

                break;
            }

            let char = c.unwrap();

            // color mark
            if char == '§' {
                let specifier = chars.nth(0).unwrap();

                // lets end previous component handling
                let mut component = ServerDescriptionComponent::default();
                component.text = txt;
                component.bold = bold;
                component.italic = italic;
                component.strikethrough = strikethrough;
                component.obfuscated = obfuscated;
                component.color = color.clone();
                component.underlined = underlined;

                // push constructed component to vector
                components.push(component);

                // parse new stuff
                if specifier == 'l' {
                    bold = Some(true);
                    style_change = true;
                }
                if specifier == 'k' {
                    obfuscated = Some(true);
                    style_change = true;
                }
                if specifier == 'm' {
                    strikethrough = Some(true);
                    style_change = true;
                }
                if specifier == 'n' {
                    underlined = Some(true);
                    style_change = true;
                }
                if specifier == 'o' {
                    italic = Some(true);
                    style_change = true;
                }

                // reset
                txt = String::new();

                if specifier.eq(&'r') {
                    bold = None;
                    italic = None;
                    strikethrough = None;
                    obfuscated = None;
                    underlined = None;
                    color = Some("white".to_owned());
                    style_change = true;
                }

                if !style_change {
                    color = Some(COLOR_INDEX.get(&specifier).unwrap().to_string());
                } else {
                    style_change = false;
                }
                continue;
            }

            txt.push(char);
        }

        components
    }

    fn generate_html(&mut self, components: &Vec<ServerDescriptionComponent>) {
        let mut text = String::new();
        for component in components {
            let mut spans = 0;

            if let Some(color) = &component.color {
                spans += 1;

                if let Some(hex) = COLOR_MAP.get(color as &str) {
                    let value = &(*hex).to_owned().clone();
                    text.push_str(&format!("<span style=\"color: {};\">", value));
                } else {
                    text.push_str(&format!("<span style=\"color: {};\">", color));
                }
            }

            if let Some(true) = component.bold {
                spans += 1;
                text.push_str("<span style=\"font-weight: bold;\">");
            }

            if let Some(true) = component.italic {
                spans += 1;
                text.push_str("<span style=\"font-style: italic;\">");
            }

            if let Some(true) = component.underlined {
                spans += 1;
                text.push_str("<span style=\"text-decoration: underline;\">");
            }

            if let Some(true) = component.strikethrough {
                spans += 1;
                text.push_str("<span style=\"text-decoration: line-through;\">");
            }

            for c in component.text.chars() {
                if c.to_string().contains("\n") {
                    text.push_str("<br>");
                } else if c.eq(&' ') {
                    text.push_str("&nbsp;");
                } else if c.eq(&'<') {
                    text.push_str("&lt;");
                } else if c.eq(&'>') {
                    text.push_str("&gt;");
                } else {
                    text.push(c);
                }
            }

            for _ in 0..spans {
                text.push_str("</span>");
            }
        }

        self.html_motd = text.clone();

        print!("{}", text);
    }

    fn generate_clean(&mut self, components: &Vec<ServerDescriptionComponent>) {
        let mut text = String::new();

        for component in components {
            text.push_str(&component.text);
        }

        self.clean_motd = text;
    }
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let mut servers = vec![];

    /*servers.push(Server {
        address: "130.61.54.151".to_owned(),
        port: Some(25565),
        minecraft_style_motd: "".to_owned(),
        html_motd: "".to_owned(),
        clean_motd: "".to_owned()
    });*/
    let mut s = Server {
        address: "51.83.170.185".to_owned(), //"80.91.223.241".to_owned(),
        port: Some(36325),                   //(25539),
        html_motd: "".to_owned(),
        clean_motd: "".to_owned(),
    };

    s.process_legacy_motd("§cCzerwony bez pogrubienia\n§lCzerwony z pogrubieniem\n§mCzerwony z czyms tam\n§6Zolty\n§rZoltyBezNiczego\n§3JkaisInnyKolor\n§lPogrubienie");

    unimplemented!();

    servers.push(&mut s);

    for server in servers {
        let mut config = ConnectionConfig::build(server.address.clone());
        if let Some(port) = server.port {
            config = config.with_port(port);
        }
        let connection = config.connect().await?;

        let response = connection.status().await?.status;
        match response.description {
            ServerDescription::Object {
                text: _text,
                extra: components,
                ..
            } => {
                server.process_motd(&components);
            }
            ServerDescription::Plain(string) => {
                println!("{}", string);
            }
        }
    }

    Ok(())
}
