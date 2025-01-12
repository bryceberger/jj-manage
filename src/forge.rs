use crate::config::Config;

pub struct NamedForge<'a> {
    pub name: &'a str,
    pub info: &'a Forge,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct Forge {
    pub url: String,
}

impl Forge {
    pub fn named<'a>(config: &'a Config, name: &'a str) -> Option<NamedForge<'a>> {
        let info = config.forges.get(name)?;
        Some(NamedForge { name, info })
    }
}
