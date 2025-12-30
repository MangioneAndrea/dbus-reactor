use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Configs {
    power_saver_mode: String,
    balanced_mode: String,
    performance_mode: String,
}

#[derive(Default)]
pub struct PowerProfileListener {
    configs: Configs,
}

impl super::Listenable for PowerProfileListener {
    const PROPERTY_NAME: &'static str = "ActiveProfile";
    const DESTINATION: &'static str = "net.hadess.PowerProfiles";
    const PATH: &'static str = "/net/hadess/PowerProfiles";
    const INTERFACE: &'static str = "net.hadess.PowerProfiles";

    async fn on_change(&self, new_value: String) {
        let mode = match new_value.as_str() {
            "power-saver" => Some(&self.configs.power_saver_mode),
            "performance" => Some(&self.configs.performance_mode),
            "balanced" => Some(&self.configs.balanced_mode),
            unknown => {
                eprintln!(
                    "THIS IS A BUG: There should be no different mode than performance | balanced | power-saver, got {unknown}"
                );
                None
            }
        };

        if let Some(mode) = mode {
            let doctor = Command::new("kscreen-doctor")
                .arg(mode)
                .env("QT_FORCE_STDERR_LOGGING", "1")
                .output();

            if let Ok(output) = doctor {
                let stderr = dbg!(String::from_utf8_lossy(&output.stderr));

                let re = regex::Regex::new(r"(?P<w>\d+)x(?P<h>\d+)@(?P<f>\d+)").unwrap();

                if let Some(caps) = re.captures(&stderr) {
                    let width = &caps["w"];
                    let height = &caps["h"];
                    let fps = &caps["f"];

                    eprintln!("Changing resolution to: {}x{}", width, height);
                    eprintln!("Changing refresh rate to: {}Hz", fps);

                    let _ = Command::new("notify-send")
                        .arg("-a")
                        .arg("dbus-reactor")
                        .arg("-i")
                        .arg("battery-low") // Icon name
                        .arg(format!(
                            "Profile Changed to {new_value}.\nResolution: {width}x{height}\nRefresh rate: {fps}Hz",
                        ))
                        .spawn();
                } else {
                    eprintln!("THIS IS A BUG: Failed to extract resolution and frame rate");
                }
            }
        }
    }
}

impl super::Configurable for PowerProfileListener {
    type Configs = Configs;
    const CONFIGS_ID: &'static str = "kde_power";

    fn new_with_config(configs: Self::Configs) -> Self {
        Self { configs }
    }

    async fn configure(self) -> Self {
        self
    }

    fn get_configs(&self) -> &Self::Configs {
        &self.configs
    }
}
