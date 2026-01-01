use std::{fmt::Display, process::Command};

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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    pub id: String,
    pub refresh_rate: f32,
    pub width: f32,
    pub height: f32,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            id,
            refresh_rate,
            width,
            height,
        } = self;

        write!(
            f,
            "{id} -- width:{width} height:{height} refresh rate:{refresh_rate}"
        )
    }
}

impl super::Configurable for PowerProfileListener {
    type Configs = Configs;
    const CONFIGS_ID: &'static str = "kde_power";

    fn new_with_config(configs: Self::Configs) -> Self {
        Self { configs }
    }

    async fn configure(self) -> Result<Self, String> {
        let kscreen_output = Command::new("kscreen-doctor")
            .arg("-j")
            .env("QT_FORCE_STDERR_LOGGING", "1")
            .output();

        let output = kscreen_output
            .map_err(|e| format!("Failed to run kscreen-doctor -j with error: {e:?}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let v: serde_json::Value =
            serde_json::from_str(&stdout).map_err(|e| format!("JSON parse error: {e:?}"))?;

        let mut modes: Vec<Mode> = v["outputs"]
            .as_array()
            .ok_or("Missing outputs array")?
            .iter()
            .flat_map(|out| {
                let out_id = out["id"].as_u64().unwrap_or(0) as usize;
                dbg!(out_id);

                out["modes"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(move |m| {
                        Some(Mode {
                            id: format!("output.{out_id}.mode.{}", m["id"].as_str()?),
                            refresh_rate: m["refreshRate"].as_f64()? as f32,
                            width: serde_json::from_value(m["size"]["width"].clone()).ok()?,
                            height: serde_json::from_value(m["size"]["height"].clone()).ok()?,
                        })
                    })
            })
            .collect();

        modes.sort_by(|a, b| {
            a.height.partial_cmp(&b.height).unwrap_or(
                a.width
                    .partial_cmp(&b.width)
                    .unwrap_or(a.refresh_rate.total_cmp(&b.refresh_rate)),
            )
        });

        modes.reverse();

        let power_saver_mode =
            inquire::Select::new("Select a mode for \"power saver\" mode", modes.clone())
                .with_starting_cursor(
                    modes
                        .iter()
                        .position(|el| el.id == self.configs.power_saver_mode)
                        .unwrap_or(0),
                )
                .with_vim_mode(true)
                .prompt()
                .map_err(|err| format!("{err:?}"))?
                .id;
        let balanced_mode =
            inquire::Select::new("Select a mode for \"balanced\" mode", modes.clone())
                .with_starting_cursor(
                    modes
                        .iter()
                        .position(|el| el.id == self.configs.balanced_mode)
                        .unwrap_or(0),
                )
                .with_vim_mode(true)
                .prompt()
                .map_err(|err| format!("{err:?}"))?
                .id;

        let performance_mode =
            inquire::Select::new("Select a mode for \"balanced\" mode", modes.clone())
                .with_starting_cursor(
                    modes
                        .iter()
                        .position(|el| el.id == self.configs.performance_mode)
                        .unwrap_or(0),
                )
                .with_vim_mode(true)
                .prompt()
                .map_err(|err| format!("{err:?}"))?
                .id;

        Ok(Self {
            configs: Configs {
                power_saver_mode,
                balanced_mode,
                performance_mode,
            },
        })
    }

    fn get_configs(&self) -> &Self::Configs {
        &self.configs
    }
}
