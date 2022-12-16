use std::str::FromStr;

use syn::Ident;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum HelperAttr {
    Track,
    Artist,
    Release,
}

impl HelperAttr {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Track => "track",
            Self::Artist => "artist",
            Self::Release => "release",
        }
    }
}

impl FromStr for HelperAttr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "track" => Ok(Self::Track),
            "artist" => Ok(Self::Artist),
            "release" => Ok(Self::Release),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Ident> for HelperAttr {
    type Error = ();

    fn try_from(value: &Ident) -> Result<Self, Self::Error> { Self::from_str(&value.to_string()) }
}
