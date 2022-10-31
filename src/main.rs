use async_minecraft_ping::{ConnectionConfig, ServerDescription, ServerError};

#[allow(unused)]
#[derive(Debug)]
struct Server {
    address: String,
    port: Option<u16>,
    motd: String,
}

impl Server {
    #[allow(unused)]
    pub fn motd(&self) -> &String {
        &self.motd
    }
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let mut servers = vec![];

    for server in servers {
        let mut config = ConnectionConfig::build(server.address);
        if let Some(port) = server.port {
            config = config.with_port(port);
        }
        let connection = config.connect().await?;

        let response = connection.status().await?.status;
        match response.description {
            ServerDescription::Object {
                text: _text,
                extra: components,
            } => {
                for component in components {
                    print!("{}", component.text);
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    Ok(())
}
