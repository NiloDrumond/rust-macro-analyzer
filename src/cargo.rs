use std::fmt;

use serde::{de::{self, MapAccess, Visitor}, Deserialize, Deserializer};

#[derive(Deserialize, Default, Debug)]
pub struct CargoTomlWorkspace {
    pub members: Vec<String>,
}

// #[derive(Deserialize, Default, Debug)]
// pub struct CargoTomlLib;

#[derive(Debug, Default)]
pub struct LibExists;

struct LibExistsVisitor;

impl<'de> Visitor<'de> for LibExistsVisitor {
    type Value = LibExists;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a [lib] section")
    }

    fn visit_map<V>(self, mut _map: V) -> Result<LibExists, V::Error>
    where
        V: MapAccess<'de>,
    {
        // We simply consume the map without processing any keys/values.
        while let Some(_key) = _map.next_key::<String>()? {
            let _ignored: de::IgnoredAny = _map.next_value()?;
        }
        Ok(LibExists)
    }
}

impl<'de> Deserialize<'de> for LibExists {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LibExistsVisitor)
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct CargoToml {
    pub workspace: Option<CargoTomlWorkspace>,
    pub lib: Option<LibExists>,
}
