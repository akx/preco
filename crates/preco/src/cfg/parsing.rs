use identify::mappings::Type;
use serde::de::Deserializer;
use serde::Deserialize;
use std::str::FromStr;
use tracing::warn;

pub(crate) fn deserialize_type_list<'de, D>(deserializer: D) -> Result<Option<Vec<Type>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    opt.map_or(Ok(None), |value: Vec<String>| {
        let mut types_vec = Vec::new();
        for typename in value {
            match Type::from_str(&typename) {
                Ok(val) => types_vec.push(val),
                Err(_) => warn!("Type `{}` is unknown and will be ignored", typename),
            }
        }
        Ok(Some(types_vec))
    })
}
