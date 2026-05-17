use crate::{
    data::{
        objects::order::Order,
        storage::data_store::{CreateOrder, DataStorage, DataStore},
    },
    utils::errors::Error,
};
use actix_web::web::{Data, Json};
use apistos::actix::CreatedJson;

#[apistos::api_operation(summary = "Add an order by table and item id.")]
pub async fn add_order(
    datastore: Data<DataStorage>,
    params: Json<CreateOrder>,
) -> Result<CreatedJson<Order>, Error> {
    let new_order = datastore
        .get_ref()
        .store
        .clone()
        .add_order(params.into_inner())
        .await?;
    Ok(CreatedJson(new_order))
}

#[apistos::api_operation(summary = "Add multiple orders by table and item id's.")]
pub async fn add_orders(
    datastore: Data<DataStorage>,
    params: Json<Vec<CreateOrder>>,
) -> Result<CreatedJson<Vec<Order>>, Error> {
    let new_orders = datastore
        .get_ref()
        .store
        .clone()
        .add_orders(params.into_inner())
        .await?;
    Ok(CreatedJson(new_orders))
}
