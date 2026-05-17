use crate::{
    data::{
        objects::order::Order,
        storage::data_store::{DataStorage, IdType, SearchOrder},
    },
    utils::errors::Error,
};
use actix_web::web::{Data, Path, Query};
use apistos::actix::CreatedJson;

#[apistos::api_operation(summary = "Remove an order by table and item id.")]
pub async fn remove_order(
    datastore: Data<DataStorage>,
    order_id: Path<IdType>,
) -> Result<CreatedJson<Option<Order>>, Error> {
    let new_order = datastore
        .get_ref()
        .store
        .clone()
        .remove_order(order_id.into_inner())
        .await?;
    Ok(CreatedJson(new_order))
}

#[apistos::api_operation(summary = "Remove multiple orders by search conditions.")]
pub async fn remove_orders(
    datastore: Data<DataStorage>,
    params: Query<SearchOrder>,
) -> Result<CreatedJson<Vec<Order>>, Error> {
    let new_orders = datastore
        .get_ref()
        .store
        .clone()
        .remove_orders(params.into_inner())
        .await?;
    Ok(CreatedJson(new_orders))
}
