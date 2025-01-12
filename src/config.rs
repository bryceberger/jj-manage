use std::{borrow::Cow, path::PathBuf};

use color_eyre::{Result, eyre::OptionExt};

use crate::{forge, get::GetConfig};

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
        fn get_user_config() -> Result<Cow<'static, str>, ()> {
            let path = xdg::BaseDirectories::with_prefix("jj-manage")
                .map_err(|e| tracing::debug!(%e, "xdg dirs error:"))?;
            let config = std::fs::read(path.get_config_file("config.toml"))
                .map_err(|e| tracing::debug!(%e, "while reading user config:"))?;
            Ok(String::from_utf8(config)
                .map_err(|e| tracing::warn!(%e, "ignoring user config:"))?
                .into())
        }
        Ok(std::iter::once(DEFAULT_CONFIG.into()).chain(get_user_config()))
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
