use crate::data::storage::data_store::IdType;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Item {
    pub id: IdType,
    pub name: String,
    pub description: String,
}
