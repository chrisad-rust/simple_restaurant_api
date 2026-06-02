use futures::FutureExt;
use rand::{RngExt, SeedableRng};

use crate::data::{
    objects::{item::Item, order::Order},
    storage::data_store::{DataStore, IdType},
};

pub struct SqliteDataStore {
    connection_pool: sqlx::SqlitePool,
}

impl SqliteDataStore {
    #[must_use]
    pub fn new() -> Result<Self, sqlx::Error> {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or(format!("sqlite:production.db"));

        Ok(Self {
            connection_pool: sqlx::SqlitePool::connect_lazy(db_url.as_str())?,
        })
    }
    #[must_use]
    pub fn from_pool(pool: sqlx::SqlitePool) -> Self {
        Self { connection_pool: pool }
    }
}

impl DataStore for SqliteDataStore {
    fn get_item<'a>(
        self: crate::utils::futures::Arc<Self>,
        item_id: IdType,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Option<crate::data::objects::item::Item>, crate::utils::errors::Error>,
    > {
        async move {
            let id = item_id as i64;
            let data = sqlx::query!("SELECT id, name, description FROM items WHERE id = ?1", id)
                .fetch_optional(&self.connection_pool)
                .await?;
            Ok(data.map(|item| Item {
                id: item.id as IdType,
                name: item.name,
                description: item.description,
            }))
        }
        .boxed()
    }

    fn get_items<'a>(
        self: crate::utils::futures::Arc<Self>,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Vec<crate::data::objects::item::Item>, crate::utils::errors::Error>,
    > {
        async move {
            let data = sqlx::query!("SELECT id, name, description FROM items ORDER BY name")
                .fetch_all(&self.connection_pool)
                .await?;
            Ok(data
                .into_iter()
                .map(|item| Item {
                    id: item.id as IdType,
                    name: item.name,
                    description: item.description,
                })
                .collect())
        }
        .boxed()
    }

    fn get_order<'a>(
        self: crate::utils::futures::Arc<Self>,
        order_id: IdType,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Option<crate::data::objects::order::Order>, crate::utils::errors::Error>,
    > {
        async move {
            let id = order_id as i64;
            let data = sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE id = ?1", id)
                .fetch_optional(&self.connection_pool)
                .await?;
            Ok(data.map(|item| Order { id: item.id as IdType, table_id: item.table_id as IdType, item_id: item.item_id as IdType, time_to_prepare: item.time_to_prepare as u8, created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) }))
        }
        .boxed()
    }

    fn get_orders<'a>(
        self: crate::utils::futures::Arc<Self>,
        args: super::data_store::SearchOrder,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Vec<crate::data::objects::order::Order>, crate::utils::errors::Error>,
    > {
        async move {
            let limit = args.limit.unwrap_or(100) as i64;
            let orders: Vec<Order> = match (args.order_state, args.table_id) {
                (crate::data::storage::data_store::SearchOrderState::Open, None) => sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE paid_at IS NULL LIMIT ?1", limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect(),
                    
                (crate::data::storage::data_store::SearchOrderState::Open, Some(table_id)) => {
                    let id = table_id as i64;
                    sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE paid_at IS NULL AND table_id = ?1 LIMIT ?2", id, limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect()
                },
                (crate::data::storage::data_store::SearchOrderState::Paid, None) => sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE paid_at IS NOT NULL LIMIT ?1", limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect(),
                (crate::data::storage::data_store::SearchOrderState::Paid, Some(table_id)) => {
                    let id = table_id as i64;
                    sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE paid_at IS NOT NULL AND table_id = ?1 LIMIT ?2", id, limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect()
                },
                (crate::data::storage::data_store::SearchOrderState::All, None) => sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders LIMIT ?1", limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect(),
                (crate::data::storage::data_store::SearchOrderState::All, Some(table_id)) => {
                      let id = table_id as i64;
                    sqlx::query!("SELECT id, table_id, item_id, time_to_prepare, created_at, paid_at FROM orders WHERE table_id = ?1 LIMIT ?2", id, limit)
                    .fetch_all(&self.connection_pool)
                    .await?.into_iter().map(|item| Order { 
                        id: item.id as IdType, 
                        table_id: item.table_id as IdType, 
                        item_id: item.item_id as IdType, 
                        time_to_prepare: item.time_to_prepare as u8, 
                        created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(item.created_at, 0).unwrap().naive_utc(), 
                        paid_at: item.paid_at.map(|item| chrono::DateTime::<chrono::Utc>::from_timestamp(item, 0).unwrap().naive_utc()) 
                    }).collect()
                },
            };
            Ok(orders)
        }.boxed()
    }

    fn add_order<'a>(
        self: crate::utils::futures::Arc<Self>,
        args: super::data_store::CreateOrder,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<crate::data::objects::order::Order, crate::utils::errors::Error>,
    > {
        async move {
            if args.table_id < 1 {
                return Err(crate::utils::errors::Error::InvalidArgument(format!("Table ID is out of range, should be higher that 1.")));
            }

            let mut conn = self.connection_pool.acquire().await?;
            let table_id = args.table_id as i64;
            let item_id = args.item_id as i64;
            let preperation_time = rand_chacha::ChaChaRng::from_seed(Default::default()).random_range(5..16) as i64;
            let created_at = chrono::Utc::now();
            let created_at_value: i64 = created_at.timestamp();
            let test = sqlx::query!("INSERT INTO orders(table_id, item_id, time_to_prepare, created_at, paid_at) VALUES (?1, ?2, ?3, ?4, ?5)", table_id, item_id, preperation_time, created_at_value, None::<i64>).execute(&mut *conn).await?;
            Ok(Order { id: test.last_insert_rowid() as u64, table_id: args.table_id, item_id: args.item_id, time_to_prepare: preperation_time as u8, created_at: chrono::DateTime::from_timestamp(created_at_value, 0).unwrap().naive_utc(), paid_at: None })
        }.boxed()
    }

    fn add_orders<'a>(
        self: crate::utils::futures::Arc<Self>,
        args: Vec<super::data_store::CreateOrder>,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Vec<crate::data::objects::order::Order>, crate::utils::errors::Error>,
    > {
        async move {
            let mut transaction = self.connection_pool.begin().await?;

            let mut orders = Vec::with_capacity(args.len());

            for arg in args.iter() {
                if arg.table_id < 1 {
                    return Err(crate::utils::errors::Error::InvalidArgument(format!("Table ID is out of range, should be higher that 1.")));
                }

                let table_id = arg.table_id as i64;
                let item_id = arg.item_id as i64;
                let preperation_time = rand_chacha::ChaChaRng::from_seed(Default::default()).random_range(5..16) as i64;
                let created_at = chrono::Utc::now();
                let created_at_value: i64 = created_at.timestamp();
                let test = sqlx::query!(
                    "INSERT INTO orders(table_id, item_id, time_to_prepare, created_at, paid_at) VALUES (?1, ?2, ?3, ?4, ?5)", 
                    table_id, item_id, preperation_time, created_at_value, None::<i64>
                ).execute(&mut *transaction).await?;
                orders.push(Order { 
                    id: test.last_insert_rowid() as u64, 
                    table_id: arg.table_id, 
                    item_id: arg.item_id, 
                    time_to_prepare: preperation_time as u8, 
                    created_at: chrono::DateTime::from_timestamp(created_at_value, 0).unwrap().naive_utc(), 
                    paid_at: None 
                });
            }
            transaction.commit().await?;

            Ok(orders)
        }.boxed()
    }

    fn pay_order<'a>(
        self: crate::utils::futures::Arc<Self>,
        order_id: u64,
    ) -> crate::utils::futures::BoxFuture<
        'a,
        Result<Option<crate::data::objects::order::Order>, crate::utils::errors::Error>,
    > {
        async move {
            let mut conn = self.connection_pool.acquire().await?;
            let id = order_id as i64;
            let paid_at = chrono::Utc::now();
            let paid_at_value = Some(paid_at.timestamp());
            sqlx::query!("UPDATE orders SET paid_at = ?1 WHERE id = ?2 AND paid_at IS NULL", paid_at_value, id).execute(&mut *conn).await?;
            return self.clone().get_order(order_id).await;
        }.boxed()
    }

    fn remove_order<'a>(
        self: crate::utils::futures::Arc<Self>,
        order_id: IdType,
    ) -> crate::utils::futures::BoxFuture<'a, Result<Option<Order>, crate::utils::errors::Error>>
    {
        async move {
            let mut conn = self.connection_pool.acquire().await?;
            let id = order_id as i64;
            let order = self.clone().get_order(order_id).await?;

            if order.is_some() {
                sqlx::query!("delete from orders WHERE id = ?1", id).execute(&mut *conn).await?;
            }
            return Ok(order);
        }.boxed()
    }

    fn remove_orders<'a>(
        self: crate::utils::futures::Arc<Self>,
        args: super::data_store::SearchOrder,
    ) -> crate::utils::futures::BoxFuture<'a, Result<Vec<Order>, crate::utils::errors::Error>>
    {
        async move {
            let orders = self.clone().get_orders(args).await?;
            let mut transaction = self.connection_pool.begin().await?;

            for order in orders.iter() {
                let id = order.id as i64;
                sqlx::query!("delete from orders WHERE id = ?1", id).execute(&mut *transaction).await?;
            }
            transaction.commit().await?;

            Ok(orders)
        }.boxed()      
    }
}

#[cfg(test)]
mod tests {
    use crate::{data::storage::data_store::{CreateOrder, SearchOrder, SearchOrderState}, utils::futures::Arc};

    use super::*;

    #[tokio::test]
    async fn test_items() {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite:test.db");
        }
        let db = Arc::new(SqliteDataStore::new().unwrap());

        // prepare test data
        {
            let mut conn = db.connection_pool.acquire().await.unwrap();
            sqlx::query!("DELETE FROM orders").execute(&mut *conn).await.unwrap();
            sqlx::query!("DELETE FROM items").execute(&mut *conn).await.unwrap();
            sqlx::query!(r#"INSERT INTO items(id, name, description) VALUES (1, "TestItem1", ""), (106, "TestItem5", "")"#).execute(&mut *conn).await.unwrap();
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
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite:test.db");
        }
        let db = Arc::new(SqliteDataStore::new().unwrap());

        // prepare test data
        {
            let mut conn = db.connection_pool.acquire().await.unwrap();
            sqlx::query!("DELETE FROM orders").execute(&mut *conn).await.unwrap();
            sqlx::query!("DELETE FROM items").execute(&mut *conn).await.unwrap();
            sqlx::query!(r#"INSERT INTO items(id, name, description) VALUES (1, "TestItem1", ""), (106, "TestItem5", "")"#).execute(&mut *conn).await.unwrap();
        }

        let added_order = db
            .clone()
            .add_order(CreateOrder {
                table_id: 4,
                item_id: 106,
            })
            .await
            .unwrap();

        assert_eq!(added_order.item_id, 106);
        assert_eq!(added_order.table_id, 4);
        assert!(added_order.time_to_prepare >= 5 || added_order.time_to_prepare <= 15);
        assert_eq!(added_order.paid_at, None);

        let added_orders = db
            .clone()
            .add_orders(vec![
                CreateOrder {
                    table_id: 4,
                    item_id: 1,
                },
                CreateOrder {
                    table_id: 10,
                    item_id: 106,
                },
            ])
            .await
            .unwrap();

        assert_eq!(added_orders.len(), 2);

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

        let paid_order = db
            .clone()
            .pay_order(added_orders.first().map(|item| item.id.clone()).unwrap())
            .await
            .unwrap();

        assert!(paid_order.is_some());

        let get_open_orders_by_table = db
            .clone()
            .get_orders(SearchOrder {
                order_state: SearchOrderState::Open,
                table_id: Some(4),
                limit: None,
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
                limit: None,
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
                limit: None,
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
                limit: None,
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
                limit: None,
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
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(get_all_orders_after_remove.len(), 1);
        assert_eq!(get_all_orders_after_remove.first(), added_orders.last());
    }
}