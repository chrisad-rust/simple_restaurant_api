use crate::{
    data::{
        objects::order::Order,
        storage::data_store::{DataStorage, DataStore, IdType},
    },
    utils::errors::Error,
};
use actix_web::web::{Data, Path};
use apistos::actix::CreatedJson;

#[apistos::api_operation(summary = "Pay an order by id.")]
pub async fn pay_order(
    datastore: Data<DataStorage>,
    order_id: Path<IdType>,
) -> Result<CreatedJson<Option<Order>>, Error> {
    let removed_order = datastore
        .get_ref()
        .store
        .clone()
        .pay_order(order_id.into_inner())
        .await?;
    Ok(CreatedJson(removed_order))
}
