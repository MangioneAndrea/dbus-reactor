mod kde_power_listener;

use std::{env, fs, path::PathBuf, sync::OnceLock};

use futures_util::StreamExt;
use kde_power_listener::PowerProfileListener;
use serde::de::DeserializeOwned;
use zbus::{Connection, Proxy, proxy::PropertyStream};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Configure the application and exit afterwards
    #[arg(short, long)]
    config: bool,
}

trait Listenable: Default {
    const PROPERTY_NAME: &'static str;
    const DESTINATION: &'static str;
    const PATH: &'static str;
    const INTERFACE: &'static str;
    async fn on_change(&self, new_value: String);

    async fn listen(self, connection: &Connection) -> anyhow::Result<()> {
        let proxy = Proxy::new(&connection, Self::DESTINATION, Self::PATH, Self::INTERFACE).await?;

        let mut changes: PropertyStream<String> =
            proxy.receive_property_changed(Self::PROPERTY_NAME).await;

        while let Some(change) = changes.next().await {
            if let Ok(value) = change.get().await {
                eprintln!("Property changed {}: {value}", Self::PROPERTY_NAME);
                self.on_change(value).await;
            }
        }

        Ok(())
    }
}

trait Configurable: Sized {
    type Configs: Default + serde::Serialize + DeserializeOwned;
    const CONFIGS_ID: &'static str;

    fn get_config_path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| {
            let config_home = env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| format!("{}/.config", env::var("HOME").unwrap_or_default()));
            PathBuf::from(config_home).join("dbus-reactor/config.toml")
        })
    }

    async fn read_configs() -> Option<Self::Configs> {
        let path = Self::get_config_path();

        let table: toml::Table = fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())?;

        table
            .get(Self::CONFIGS_ID)
            .cloned()
            .and_then(|value| value.try_into().ok())?
    }

    async fn persist(&self) {
        let configs = self.get_configs();
        let path = Self::get_config_path();

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut table: toml::Table = fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default();

        if let Ok(value) = toml::Value::try_from(configs) {
            table.insert(Self::CONFIGS_ID.to_string(), value);
            if let Ok(serialized) = toml::to_string_pretty(&table) {
                let _ = fs::write(path, serialized);
            }
        }
    }

    fn get_configs(&self) -> &Self::Configs;

    async fn configure(self) -> Result<Self, String>;

    fn new_with_config(configs: Self::Configs) -> Self;

    async fn new() -> Self {
        Self::new_with_config(Self::read_configs().await.unwrap_or_default())
    }
}

async fn run(_: Args) {
    let connection = Box::leak(Box::new(
        Connection::system()
            .await
            .expect("Attaching to dbus should always be possible"),
    ));

    let mut js = tokio::task::JoinSet::new();

    let power_profile_listener = PowerProfileListener::new().await;
    js.spawn(power_profile_listener.listen(connection));

    js.join_all().await;
}

async fn config(_: Args) {
    PowerProfileListener::new()
        .await
        .configure()
        .await
        .unwrap()
        .persist()
        .await;
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.config {
        config(args).await;
    } else {
        run(args).await;
    }
}
