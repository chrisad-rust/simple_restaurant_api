use crate::{
    data::{
        objects::order::Order,
        storage::data_store::{DataStorage, DataStore, IdType, SearchOrder},
    },
    utils::errors::Error,
};
use actix_web::web::{Data, Path, Query};
use apistos::actix::CreatedJson;

#[apistos::api_operation(summary = "Get an order by id.")]
pub async fn get_order(
    datastore: Data<DataStorage>,
    order_id: Path<IdType>,
) -> Result<CreatedJson<Option<Order>>, Error> {
    let order = datastore
        .get_ref()
        .store
        .clone()
        .get_order(order_id.into_inner())
        .await?;
    Ok(CreatedJson(order))
}

#[apistos::api_operation(summary = "Get orders by serach parameters.")]
pub async fn get_orders(
    datastore: Data<DataStorage>,
    params: Query<SearchOrder>,
) -> Result<CreatedJson<Vec<Order>>, Error> {
    let orders = datastore
        .get_ref()
        .store
        .clone()
        .get_orders(params.into_inner())
        .await?;
    Ok(CreatedJson(orders))
}
