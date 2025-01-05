use std::{borrow::Cow, path::PathBuf};

use color_eyre::{Result, eyre::OptionExt};

use crate::forge;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    base: String,
    #[serde(default = "whoami::username")]
    pub user: String,
    pub default_forge: forge::Forge,
    pub colocate: bool,

    pub get: GetConfig,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct GetConfig {
    pub clone_kind: CloneKind,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CloneKind {
    #[default]
    Ssh,
    Https,
}

pub const DEFAULT_CONFIG: &str = r#"
base = "repos"
default-forge = "github"
colocate = true

[get]
clone-kind = "ssh"
"#;

impl Config {
    pub fn realize(configs: impl IntoIterator<Item: AsRef<str>>) -> Result<Self> {
        let mut config = toml::Table::new();
        for s in configs {
            let layer: toml::Table = s.as_ref().parse()?;
            deep_update(&mut config, layer);
        }
        Ok(config.try_into()?)
    }

    pub fn default_layers() -> Result<impl Iterator<Item = Cow<'static, str>>> {
        let user_config_path =
            xdg::BaseDirectories::with_prefix("jj-manage")?.get_config_file("config.toml");
        let user_config = std::fs::read(user_config_path)
            .map_err(|e| tracing::debug!(%e, "no user config:"))
            .ok()
            .and_then(|s| {
                String::from_utf8(s)
                    .map_err(|e| tracing::warn!(%e, "ignoring user config:"))
                    .ok()
                    .map(Cow::from)
            });
        Ok(std::iter::once(DEFAULT_CONFIG.into()).chain(user_config))
    }

    pub fn base(&self) -> Result<PathBuf> {
        let mut base = home::home_dir().ok_or_eyre("could not determine home dir")?;
        base.push(&self.base);
        Ok(base)
    }
}

fn deep_update(base: &mut toml::Table, right: toml::Table) {
    for (key, val) in right {
        match base.entry(key) {
            toml::map::Entry::Vacant(ent) => {
                ent.insert(val);
            }
            toml::map::Entry::Occupied(ent) => match val {
                toml::Value::Table(map) => {
                    let ent = ent.into_mut();
                    if let toml::Value::Table(b) = ent {
                        deep_update(b, map);
                    } else {
                        *ent = toml::Value::Table(map);
                    }
                }
                _ => *ent.into_mut() = val,
            },
        }
    }
}
