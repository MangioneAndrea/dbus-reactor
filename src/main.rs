mod kde_power_listener;

use futures_util::StreamExt;
use kde_power_listener::PowerProfileListener;
use zbus::{Connection, Proxy, proxy::PropertyStream};

trait Listenable: Default {
    const PROPERTY_NAME: &'static str;
    const DESTINATION: &'static str;
    const PATH: &'static str;
    const INTERFACE: &'static str;
    async fn on_change(&self, new_value: String);

    async fn listen(&self, connection: &Connection) -> anyhow::Result<()> {
        let proxy = Proxy::new(&connection, Self::DESTINATION, Self::PATH, Self::INTERFACE).await?;

        let mut changes: PropertyStream<String> =
            proxy.receive_property_changed(Self::PROPERTY_NAME).await;

        while let Some(change) = changes.next().await {
            if let Ok(value) = change.get().await {
                self.on_change(value).await;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let connection = Box::leak(Box::new(
        Connection::system()
            .await
            .expect("Attaching to dbus should always be possible"),
    ));

    let mut js = tokio::task::JoinSet::new();

    js.spawn(PowerProfileListener.listen(connection));

    js.join_all().await;
}
