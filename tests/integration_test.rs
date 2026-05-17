use actix_web::web::{Data, delete, get, patch, post, resource};
use simple_restaurant_api::{
    data::{
        objects::order::Order,
        storage::{
            data_store::{CreateOrder, DataStorage, IdType},
            sqlite_data_store::SqliteDataStore,
        },
    },
    services::{self},
    utils::futures::Arc,
};

#[actix_web::test]
async fn test_services() {
    unsafe {
        std::env::set_var("DATABASE_URL", "sqlite:test.db");
    }
    let db_pool = sqlx::SqlitePool::connect_lazy("sqlite:test.db").unwrap();

    // prepare test data
    {
        let mut conn = db_pool.acquire().await.unwrap();
        sqlx::query!("DELETE FROM orders")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM items")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query!(r#"INSERT INTO items(id, name, description) VALUES (1, "TestItem1", ""), (106, "TestItem5", "")"#).execute(&mut *conn).await.unwrap();
    }

    let datastore = DataStorage {
        store: Arc::new(SqliteDataStore::from_pool(db_pool)),
    };

    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(Data::new(datastore))
            .service(resource("/items/{item_id}").route(get().to(services::get_items::get_item)))
            .service(resource("/items").route(get().to(services::get_items::get_items)))
            .service(
                resource("/orders/{order_id}")
                    .route(get().to(services::get_orders::get_order))
                    .route(patch().to(services::pay_orders::pay_order))
                    .route(delete().to(services::remove_orders::remove_order)),
            )
            .service(
                resource("/orders")
                    .route(get().to(services::get_orders::get_orders))
                    .route(post().to(services::add_orders::add_orders))
                    .route(delete().to(services::remove_orders::remove_orders)),
            )
            .service(resource("/order").route(post().to(services::add_orders::add_order))),
    )
    .await;

    let added_order_req = actix_web::test::TestRequest::post()
        .uri("/order")
        .set_json(CreateOrder {
            table_id: 4,
            item_id: 106,
        })
        .to_request();
    let added_order: Order = actix_web::test::call_and_read_body_json(&app, added_order_req).await;

    let added_orders_req = actix_web::test::TestRequest::post()
        .uri("/orders")
        .set_json(vec![
            CreateOrder {
                table_id: 4,
                item_id: 1,
            },
            CreateOrder {
                table_id: 10,
                item_id: 106,
            },
        ])
        .to_request();
    let added_orders: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, added_orders_req).await;

    assert_eq!(added_orders.len(), 2);

    let invalid_add_order_req = actix_web::test::TestRequest::post()
        .uri("/order")
        .set_json(CreateOrder {
            table_id: 0,
            item_id: 10,
        })
        .to_request();
    let invalid_add_order_resp = actix_web::test::call_service(&app, invalid_add_order_req).await;

    assert!(invalid_add_order_resp.status() == actix_web::http::StatusCode::BAD_REQUEST);

    let invalid_add_orders_req = actix_web::test::TestRequest::post()
        .uri("/orders")
        .set_json(vec![
            CreateOrder {
                table_id: 10,
                item_id: 10,
            },
            CreateOrder {
                table_id: 10,
                item_id: 55,
            },
        ])
        .to_request();
    let invalid_add_orders_resp = actix_web::test::call_service(&app, invalid_add_orders_req).await;
    assert!(invalid_add_orders_resp.status() == actix_web::http::StatusCode::CONFLICT);

    let pay_url = format!(
        "/orders/{}",
        added_orders.first().map(|item| item.id.clone()).unwrap()
    );
    let pay_req = actix_web::test::TestRequest::patch()
        .uri(&pay_url)
        .to_request();
    let paid_order: Option<Order> = actix_web::test::call_and_read_body_json(&app, pay_req).await;

    assert!(paid_order.is_some());

    let get_open_orders_by_table_url = format!("/orders?table_id=4&order_state=Open");
    let get_open_orders_by_table_req = actix_web::test::TestRequest::get()
        .uri(&get_open_orders_by_table_url)
        .to_request();
    let get_open_orders_by_table: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, get_open_orders_by_table_req).await;

    assert_eq!(get_open_orders_by_table.len(), 1);
    assert_eq!(get_open_orders_by_table.first(), Some(&added_order));

    let get_paid_orders_by_table_url = format!("/orders?table_id=4&order_state=Paid");
    let get_paid_orders_by_table_req = actix_web::test::TestRequest::get()
        .uri(&get_paid_orders_by_table_url)
        .to_request();
    let get_paid_orders_by_table: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, get_paid_orders_by_table_req).await;

    assert_eq!(get_paid_orders_by_table.len(), 1);
    assert_eq!(get_paid_orders_by_table.first(), paid_order.as_ref());

    let get_all_orders_by_table_url = format!("/orders?table_id=4&order_state=All");
    let get_all_orders_by_table_req = actix_web::test::TestRequest::get()
        .uri(&get_all_orders_by_table_url)
        .to_request();
    let get_all_orders_by_table: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, get_all_orders_by_table_req).await;

    assert_eq!(get_all_orders_by_table.len(), 2);
    assert_eq!(get_all_orders_by_table.first(), Some(&added_order));
    assert_eq!(get_paid_orders_by_table.last(), paid_order.as_ref());

    let removed_order_url = format!("/orders/{}", added_order.id);
    let removed_order_req = actix_web::test::TestRequest::delete()
        .uri(&removed_order_url)
        .to_request();
    let removed_order: Option<Order> =
        actix_web::test::call_and_read_body_json(&app, removed_order_req).await;

    assert!(removed_order.is_some());

    let get_all_orders_after_remove_url = format!("/orders?order_state=All");
    let get_all_orders_after_remove_req = actix_web::test::TestRequest::get()
        .uri(&get_all_orders_after_remove_url)
        .to_request();
    let get_all_orders_after_remove: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, get_all_orders_after_remove_req).await;

    assert_eq!(get_all_orders_after_remove.len(), 2);
    assert_eq!(get_all_orders_after_remove.first(), paid_order.as_ref());
    assert_eq!(get_all_orders_after_remove.last(), added_orders.last());

    let removed_table_orders_url = format!("/orders?table_id=4&order_state=All");
    let removed_table_orders_req = actix_web::test::TestRequest::delete()
        .uri(&removed_table_orders_url)
        .to_request();
    let removed_table_orders: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, removed_table_orders_req).await;

    assert_eq!(removed_table_orders.len(), 1);
    assert_eq!(removed_table_orders.first(), paid_order.as_ref());

    let get_all_orders_after_remove_url = format!("/orders?order_state=All");
    let get_all_orders_after_remove_req = actix_web::test::TestRequest::get()
        .uri(&get_all_orders_after_remove_url)
        .to_request();
    let get_all_orders_after_remove: Vec<Order> =
        actix_web::test::call_and_read_body_json(&app, get_all_orders_after_remove_req).await;

    assert_eq!(get_all_orders_after_remove.len(), 1);
    assert_eq!(get_all_orders_after_remove.first(), added_orders.last());
}

#[actix_web::test]
async fn test_services_with_multiple_clients() {
    unsafe {
        std::env::set_var("DATABASE_URL", "sqlite:test.db");
    }
    let db_pool = sqlx::SqlitePool::connect_lazy("sqlite:test.db").unwrap();

    // prepare test data
    {
        let mut conn = db_pool.acquire().await.unwrap();
        sqlx::query!("DELETE FROM orders")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM items")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query!(r#"INSERT INTO items(id, name, description) VALUES (1, "TestItem1", ""), (106, "TestItem5", "")"#).execute(&mut *conn).await.unwrap();
    }

    let datastore = DataStorage {
        store: Arc::new(SqliteDataStore::from_pool(db_pool)),
    };

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(Data::new(datastore.clone()))
            .service(resource("/items/{item_id}").route(get().to(services::get_items::get_item)))
            .service(resource("/items").route(get().to(services::get_items::get_items)))
            .service(
                resource("/orders/{order_id}")
                    .route(get().to(services::get_orders::get_order))
                    .route(patch().to(services::pay_orders::pay_order))
                    .route(delete().to(services::remove_orders::remove_order)),
            )
            .service(
                resource("/orders")
                    .route(get().to(services::get_orders::get_orders))
                    .route(post().to(services::add_orders::add_orders))
                    .route(delete().to(services::remove_orders::remove_orders)),
            )
            .service(resource("/order").route(post().to(services::add_orders::add_order)))
    })
    .bind(("127.0.0.1", 8888))
    .unwrap()
    .run();
    let server_handle = server.handle();
    let server_join_handle = actix_web::rt::spawn(server);

    async fn add_orders(table_id: IdType, item_id: IdType) {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        for _ in 0..10 {
            let resp = client
                .post("http://localhost:8888/order")
                .json(&CreateOrder {
                    table_id: table_id,
                    item_id: item_id,
                })
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
        }
    }

    tokio::join!(
        add_orders(1, 1),
        add_orders(2, 106),
        add_orders(3, 1),
        add_orders(4, 106),
        add_orders(5, 1),
        add_orders(6, 106),
        add_orders(7, 1),
        add_orders(8, 106),
        add_orders(9, 1),
        add_orders(10, 106)
    );

    async fn get_orders() {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        let resp = client
            .get("http://localhost:8888/orders?order_state=All")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
        let orders: Vec<Order> = resp.json().await.unwrap();
        assert_eq!(orders.len(), 100);
    }

    tokio::join!(get_orders(), get_orders(), get_orders());

    server_handle.stop(true).await;
    server_join_handle.await.unwrap().unwrap();
}
