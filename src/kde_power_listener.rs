use std::process::Command;

#[derive(Default)]
pub struct PowerProfileListener;

impl super::Listenable for PowerProfileListener {
    const PROPERTY_NAME: &'static str = "ActiveProfile";
    const DESTINATION: &'static str = "net.hadess.PowerProfiles";
    const PATH: &'static str = "/net/hadess/PowerProfiles";
    const INTERFACE: &'static str = "net.hadess.PowerProfiles";

    async fn on_change(&self, new_value: String) {
        match new_value.as_str() {
            "power-saver" => {
                let _ = Command::new("kscreen-doctor")
                    .arg("output.1.mode.2")
                    .spawn();
            }
            "balanced" | "performance" => {
                let _ = Command::new("kscreen-doctor")
                    .arg("output.1.mode.1")
                    .spawn();
            }
            _ => {}
        }
    }
}
