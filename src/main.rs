use actix_web::{App, HttpServer, web::Data};
use apistos::{
    SwaggerUIConfig,
    app::{BuildConfig, OpenApiWrapper},
    info::Info,
    spec::Spec,
    web::{delete, get, patch, post, resource, scope},
};
use simple_restaurant_api::{
    data::storage::{data_store::DataStorage, sqlite_data_store::SqliteDataStore},
    services,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_level(true)
        .with_max_level(tracing::Level::ERROR)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Subscriber should be configurable!");
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("error"));

    let datastore = DataStorage {
        store: simple_restaurant_api::utils::futures::Arc::new(
            SqliteDataStore::new().expect("Database should be exist."),
        ),
    };

    HttpServer::new(move || {
        let api_spec = Spec {
            info: Info {
                title: format!("Simple restaurant API."),
                ..Default::default()
            },
            ..Default::default()
        };
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(actix_web::middleware::Logger::default().log_level(log::Level::Debug))
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .send_wildcard(),
            )
            .document(api_spec)
            .app_data(Data::new(datastore.clone()))
            .service(
                scope("/api/v1")
                    .service(
                        resource("/items/{item_id}").route(get().to(services::get_items::get_item)),
                    )
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
            .build_with(
                "/docs/openapi.json",
                BuildConfig::default().with(SwaggerUIConfig::new(&"/docs")),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
