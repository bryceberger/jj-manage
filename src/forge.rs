use crate::config::Config;

pub struct NamedForge<'a> {
    pub name: &'a str,
    pub info: &'a Forge,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Forge {
    pub url: String,
    pub clone_kind: Option<CloneKind>,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CloneKind {
    #[default]
    Ssh,
    Https,
}

impl Forge {
    pub fn named<'a>(config: &'a Config, name: &'a str) -> Option<NamedForge<'a>> {
        let info = config.forges.get(name)?;
        Some(NamedForge { name, info })
    }
}
