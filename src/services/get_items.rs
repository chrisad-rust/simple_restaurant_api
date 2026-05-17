use crate::{
    data::{
        objects::item::Item,
        storage::data_store::{DataStorage, IdType},
    },
    utils::errors::Error,
};
use actix_web::web::{Data, Path};
use apistos::actix::CreatedJson;

#[apistos::api_operation(summary = "Get an item by id.")]
pub async fn get_item(
    datastore: Data<DataStorage>,
    item_id: Path<IdType>,
) -> Result<CreatedJson<Option<Item>>, Error> {
    let item = datastore
        .get_ref()
        .store
        .clone()
        .get_item(item_id.into_inner())
        .await?;
    Ok(CreatedJson(item))
}

#[apistos::api_operation(summary = "Get all items.")]
pub async fn get_items(datastore: Data<DataStorage>) -> Result<CreatedJson<Vec<Item>>, Error> {
    let items = datastore.get_ref().store.clone().get_items().await?;
    Ok(CreatedJson(items))
}
