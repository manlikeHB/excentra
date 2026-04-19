use crate::utils::query_builder::QueryOrder;
use serde::Deserialize;

pub fn deserialize_order<'de, D>(deserializer: D) -> Result<Option<QueryOrder>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s.as_deref().map(|s| s.to_lowercase()).as_deref() {
        Some("asc") => Ok(Some(QueryOrder::ASC)),
        Some("desc") => Ok(Some(QueryOrder::DESC)),
        None => Ok(None),
        Some(other) => Err(serde::de::Error::custom(format!(
            "unknown status: {}",
            other
        ))),
    }
}
