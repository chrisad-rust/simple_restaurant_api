use crate::data::objects::item::Item;
use crate::data::objects::order::Order;
use crate::utils::errors::Error;
use crate::utils::futures::{Arc, BoxFuture};

pub type IdType = u64;

#[derive(
    Default, serde::Serialize, serde::Deserialize,
)]
pub enum SearchOrderState {
    #[default]
    Open = 0,
    Paid = 1,
    All = 2,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchOrder {
    pub order_state: SearchOrderState,
    pub table_id: Option<IdType>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateOrder {
    pub table_id: IdType,
    pub item_id: IdType,
}

pub trait DataStore: 'static + Send + Sync {
    #[must_use]
    fn get_item<'a>(self: Arc<Self>, item_id: IdType)
    -> BoxFuture<'a, Result<Option<Item>, Error>>;
    #[must_use]
    fn get_items<'a>(self: Arc<Self>) -> BoxFuture<'a, Result<Vec<Item>, Error>>;

    #[must_use]
    fn get_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>>;
    #[must_use]
    fn get_orders<'a>(
        self: Arc<Self>,
        args: SearchOrder,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>>;

    #[must_use]
    fn add_order<'a>(self: Arc<Self>, args: CreateOrder) -> BoxFuture<'a, Result<Order, Error>>;

    #[must_use]
    fn add_orders<'a>(
        self: Arc<Self>,
        args: Vec<CreateOrder>,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>>;

    #[must_use]
    fn paid_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>>;

    #[must_use]
    fn remove_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>>;

    #[must_use]
    fn remove_orders<'a>(
        self: Arc<Self>,
        args: SearchOrder,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>>;
}

#[derive(Clone)]
pub struct DataStorage {
    pub store: Arc<dyn DataStore>,
}
