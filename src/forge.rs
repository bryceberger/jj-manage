use std::borrow::Cow;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Forge {
    #[default]
    Github,
    Gitlab,
    Custom(Box<ForgeInfo>),
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct ForgeInfo {
    pub url: Cow<'static, str>,
}

impl Forge {
    pub fn get_info(&self) -> &ForgeInfo {
        match self {
            Forge::Github => &ForgeInfo {
                url: Cow::Borrowed("github.com"),
            },
            Forge::Gitlab => &ForgeInfo {
                url: Cow::Borrowed("gitlab.com"),
            },
            Forge::Custom(forge_info) => forge_info,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Forge::Github => "github",
            Forge::Gitlab => "gitlab",
            Forge::Custom(forge_info) => &forge_info.url,
        }
    }

    pub fn from_str(input: impl Into<Cow<'static, str>> + AsRef<str>) -> Self {
        let input_str = input.as_ref();
        if input_str.eq_ignore_ascii_case("github") {
            Forge::Github
        } else if input_str.eq_ignore_ascii_case("gitlab") {
            Forge::Gitlab
        } else {
            Forge::Custom(Box::new(ForgeInfo { url: input.into() }))
        }
    }
}
