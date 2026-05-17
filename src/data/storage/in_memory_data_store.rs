use std::collections::BTreeMap;

use futures::FutureExt;
use tokio::sync::RwLock;

use crate::data::objects::item::Item;
use crate::data::objects::order::Order;
use crate::data::storage::data_store::{DataStore, IdType};
use crate::utils::errors::Error;
use crate::utils::futures::{Arc, BoxFuture};

#[derive(Clone)]
pub struct InMemoryDataStore {
    items: Arc<RwLock<BTreeMap<IdType, Item>>>,
    orders: Arc<RwLock<BTreeMap<IdType, Order>>>,
}

impl InMemoryDataStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(BTreeMap::new())),
            orders: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl DataStore for InMemoryDataStore {
    fn get_item<'a>(
        self: Arc<Self>,
        item_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Item>, Error>> {
        async move {
            let items_guard = self.items.read().await;
            Ok(items_guard.get(&item_id).cloned())
        }
        .boxed()
    }

    fn get_items<'a>(self: Arc<Self>) -> BoxFuture<'a, Result<Vec<Item>, Error>> {
        async move {
            let items_guard = self.items.read().await;
            Ok(items_guard.values().cloned().collect())
        }
        .boxed()
    }

    fn get_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>> {
        async move {
            let orders_guard = self.orders.read().await;
            Ok(orders_guard.get(&order_id).cloned())
        }
        .boxed()
    }

    fn get_orders<'a>(
        self: Arc<Self>,
        args: super::data_store::SearchOrder,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>> {
        async move {
            let orders_guard = self.orders.read().await;
            let orders_iter = orders_guard
                .values()
                .filter(|item| match &args.order_state {
                    super::data_store::SearchOrderState::Open => item.paid_at.is_none(),
                    super::data_store::SearchOrderState::Paid => item.paid_at.is_some(),
                    super::data_store::SearchOrderState::All => true,
                });
            if let Some(table_id) = args.table_id {
                return Ok(orders_iter
                    .filter(|item| item.table_id == table_id)
                    .cloned()
                    .collect());
            }
            Ok(orders_iter.cloned().collect())
        }
        .boxed()
    }

    fn add_order<'a>(
        self: Arc<Self>,
        args: super::data_store::CreateOrder,
    ) -> BoxFuture<'a, Result<Order, Error>> {
        async move {
            if args.table_id < 1 {
                return Err(crate::utils::errors::Error::InvalidArgument(format!(
                    "Table ID is out of range, should be higher that 1."
                )));
            }
            let items_guard = self.items.read().await;

            if !items_guard.contains_key(&args.item_id) {
                return Err(crate::utils::errors::Error::Conflict(format!(
                    "Item does not exist with id {}.",
                    args.item_id
                )));
            }

            let mut orders_guard = self.orders.write().await;
            let new_order = Order {
                id: orders_guard.len() as IdType,
                table_id: args.table_id,
                item_id: args.item_id,
                time_to_prepare: 0,
                created_at: chrono::Utc::now().naive_utc(),
                paid_at: None,
            };
            orders_guard.insert(new_order.id, new_order.clone());
            Ok(new_order)
        }
        .boxed()
    }

    fn add_orders<'a>(
        self: Arc<Self>,
        args: Vec<super::data_store::CreateOrder>,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>> {
        async move {
            let items_guard = self.items.read().await;
            let mut orders_guard = self.orders.write().await;

            for arg in args.iter() {
                if arg.table_id < 1 {
                    return Err(crate::utils::errors::Error::Conflict(format!(
                        "Table ID is out of range, should be higher that 1."
                    )));
                }

                if !items_guard.contains_key(&arg.item_id) {
                    return Err(crate::utils::errors::Error::Conflict(format!(
                        "Item does not exist with id {}.",
                        arg.item_id
                    )));
                }
            }

            let mut new_orders = Vec::with_capacity(args.len());

            for arg in args {
                let new_order = Order {
                    id: orders_guard.len() as IdType,
                    table_id: arg.table_id,
                    item_id: arg.item_id,
                    time_to_prepare: 0,
                    created_at: chrono::Utc::now().naive_utc(),
                    paid_at: None,
                };
                orders_guard.insert(new_order.id, new_order.clone());
                new_orders.push(new_order);
            }

            Ok(new_orders)
        }
        .boxed()
    }

    fn paid_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>> {
        async move {
            let mut orders_guard = self.orders.write().await;

            if let Some(order) = orders_guard.get_mut(&order_id) {
                order.paid_at = Some(chrono::Utc::now().naive_utc());
                return Ok(Some(order.clone()));
            }
            Ok(None)
        }
        .boxed()
    }

    fn remove_order<'a>(
        self: Arc<Self>,
        order_id: IdType,
    ) -> BoxFuture<'a, Result<Option<Order>, Error>> {
        async move {
            let mut orders_guard = self.orders.write().await;
            Ok(orders_guard.remove(&order_id))
        }
        .boxed()
    }

    fn remove_orders<'a>(
        self: Arc<Self>,
        args: super::data_store::SearchOrder,
    ) -> BoxFuture<'a, Result<Vec<Order>, Error>> {
        async move {
            let mut orders_guard = self.orders.write().await;
            let mut removed_items = Vec::with_capacity(orders_guard.len());

            orders_guard.retain(|_, value| {
                let remove_by_table_cond = args
                    .table_id
                    .as_ref()
                    .is_some_and(|table_id| value.table_id.eq(table_id));
                let remove_by_state_cond = match args.order_state {
                    super::data_store::SearchOrderState::Open => value.paid_at.is_none(),
                    super::data_store::SearchOrderState::Paid => value.paid_at.is_some(),
                    super::data_store::SearchOrderState::All => true,
                };

                if remove_by_state_cond && remove_by_table_cond {
                    removed_items.push(value.clone());
                    return false;
                }
                return true;
            });
            Ok(removed_items)
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::storage::data_store::{CreateOrder, SearchOrder, SearchOrderState};

    use super::*;

    #[tokio::test]
    async fn test_items() {
        let db = Arc::new(InMemoryDataStore::new());

        // write test data
        {
            let mut items_guard = db.items.write().await;
            items_guard.insert(
                1,
                Item {
                    id: 1,
                    name: format!("TestItem1"),
                    description: format!(""),
                },
            );
            items_guard.insert(
                106,
                Item {
                    id: 106,
                    name: format!("TestItem4"),
                    description: format!(""),
                },
            );
            items_guard.insert(
                106,
                Item {
                    id: 106,
                    name: format!("TestItem5"),
                    description: format!(""),
                },
            );
        }

        assert!(
            db.clone()
                .get_item(1)
                .await
                .is_ok_and(|item| item.is_some_and(|item| item.name.as_str() == "TestItem1"))
        );
        assert!(
            db.clone()
                .get_item(2)
                .await
                .is_ok_and(|item| item.is_none())
        );
        assert!(
            db.clone()
                .get_item(106)
                .await
                .is_ok_and(|item| item.is_some_and(|item| item.name.as_str() == "TestItem5"))
        );

        let items = db.clone().get_items().await.unwrap();

        assert_eq!(items.len(), 2);
        assert!(
            items
                .get(0)
                .is_some_and(|item| item.name.as_str() == "TestItem1")
        );
        assert!(
            items
                .get(1)
                .is_some_and(|item| item.name.as_str() == "TestItem5")
        );
        assert!(items.get(2).is_none());
    }

    #[tokio::test]
    async fn test_orders() {
        let db = Arc::new(InMemoryDataStore::new());

        // write test data
        {
            let mut items_guard = db.items.write().await;
            items_guard.insert(
                10,
                Item {
                    id: 10,
                    name: format!("TestItem1"),
                    description: format!(""),
                },
            );
        }

        let added_order = db
            .clone()
            .add_order(CreateOrder {
                table_id: 4,
                item_id: 10,
            })
            .await
            .unwrap();
        let added_orders = db
            .clone()
            .add_orders(vec![
                CreateOrder {
                    table_id: 4,
                    item_id: 10,
                },
                CreateOrder {
                    table_id: 10,
                    item_id: 10,
                },
            ])
            .await
            .unwrap();
        assert!(
            db.clone()
                .add_order(CreateOrder {
                    table_id: 0,
                    item_id: 10
                })
                .await
                .is_err()
        );
        assert!(
            db.clone()
                .add_orders(vec![
                    CreateOrder {
                        table_id: 10,
                        item_id: 10,
                    },
                    CreateOrder {
                        table_id: 10,
                        item_id: 55,
                    }
                ])
                .await
                .is_err()
        );

        assert_eq!(added_orders.len(), 2);

        let paid_order = db
            .clone()
            .paid_order(added_orders.first().map(|item| item.id.clone()).unwrap())
            .await
            .unwrap();

        assert!(paid_order.is_some());

        let get_open_orders_by_table = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::Open,
                table_id: Some(4),
            })
            .await
            .unwrap();

        assert_eq!(get_open_orders_by_table.len(), 1);
        assert_eq!(get_open_orders_by_table.first(), Some(&added_order));

        let get_paid_orders_by_table = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::Paid,
                table_id: Some(4),
            })
            .await
            .unwrap();

        assert_eq!(get_paid_orders_by_table.len(), 1);
        assert_eq!(get_paid_orders_by_table.first(), paid_order.as_ref());

        let get_all_orders_by_table = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::All,
                table_id: Some(4),
            })
            .await
            .unwrap();

        assert_eq!(get_all_orders_by_table.len(), 2);
        assert_eq!(get_all_orders_by_table.first(), Some(&added_order));
        assert_eq!(get_paid_orders_by_table.last(), paid_order.as_ref());

        let removed_order = db.clone().remove_order(added_order.id).await.unwrap();

        assert!(removed_order.is_some());

        let get_all_orders_after_remove = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::All,
                table_id: None,
            })
            .await
            .unwrap();

        assert_eq!(get_all_orders_after_remove.len(), 2);
        assert_eq!(get_all_orders_after_remove.first(), paid_order.as_ref());
        assert_eq!(get_all_orders_after_remove.last(), added_orders.last());

        let removed_table_orders = db
            .clone()
            .remove_orders(SearchOrder {
                order_state: SearchOrderState::All,
                table_id: Some(4),
            })
            .await
            .unwrap();

        assert_eq!(removed_table_orders.len(), 1);
        assert_eq!(removed_table_orders.first(), paid_order.as_ref());

        let get_all_orders_after_remove = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::All,
                table_id: None,
            })
            .await
            .unwrap();

        assert_eq!(get_all_orders_after_remove.len(), 1);
        assert_eq!(get_all_orders_after_remove.first(), added_orders.last());
    }
}
