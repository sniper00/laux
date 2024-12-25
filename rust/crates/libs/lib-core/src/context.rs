use dashmap::DashMap;
use lazy_static::lazy_static;
use reqwest::ClientBuilder;
use std::time::Duration;
use tokio::runtime::Builder;

lazy_static! {
    pub static ref CONTEXT: Context = {
        let tokio_runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_time()
            .enable_io()
            .build();

        Context {
            http_clients: DashMap::new(),
            tokio_runtime: tokio_runtime.expect("Init tokio runtime failed")
        }
    };
}

pub struct Context {
    http_clients: DashMap<String, reqwest::Client>,
    pub tokio_runtime: tokio::runtime::Runtime,
}

impl Context {
    pub fn get_http_client(&self, timeout: u64, proxy: &String) -> reqwest::Client {
        let name = format!("{}_{}", timeout, proxy);
        if let Some(client) = self.http_clients.get(&name) {
            return client.clone();
        }

        let builder = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .use_rustls_tls()
            .tcp_nodelay(true);

        let client = if proxy.is_empty() {
            builder.build().unwrap_or_default()
        } else {
            match reqwest::Proxy::all(proxy) {
                Ok(proxy) => builder.proxy(proxy).build().unwrap_or_default(),
                Err(_) => builder.build().unwrap_or_default(),
            }
        };

        self.http_clients.insert(name.to_string(), client.clone());
        client
    }
}
