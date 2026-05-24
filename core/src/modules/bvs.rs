use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Server {
    protocol: String,
    host: String,
    port: u16,
    path: String,
}

impl Server {
    pub fn get_protocol(&self) -> String {
        self.protocol.clone()
    }

    pub fn get_host(&self) -> String {
        self.host.clone()
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn new(protocol: String, host: String, port: u16, path: String) -> Self {
        Self {
            protocol,
            host,
            port,
            path,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BvsConfig {
    version: String,
    server: Server,
}

impl BvsConfig {
    pub fn new(version: String, server: Server) -> Self {
        Self { version, server }
    }

    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    pub fn get_server(&self) -> Server {
        self.server.clone()
    }
}
