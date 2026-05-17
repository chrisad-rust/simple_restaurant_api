use crate::data::storage::data_store::IdType;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    apistos::ApiComponent,
    schemars::JsonSchema,
)]
pub struct Order {
    pub id: IdType,
    pub table_id: IdType,
    pub item_id: IdType,
    pub time_to_prepare: u8,
    pub created_at: chrono::NaiveDateTime,
    pub paid_at: Option<chrono::NaiveDateTime>,
}
