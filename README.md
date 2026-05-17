## System Actors

### The application
- Actix_web implementation with apistos for openapi support.
- sqlite for database to keep the setup slim. 

### The client 
- Because of the openapi implementation is every openapi client viable.
- If I have the time over, I will implement one by myself.

## Requirements

| Requirement | Solution |
|--------------|------------|
| The server API MUST fully follow REST API principles and present a set of HTTP endpoints to connect to. | Ensure this by using openapi and schema validation tools. |
| The client (the restaurant staff “devices” making the requests) MUST be able to: add one or more items with a table number, remove an item for a table, and query the items still remaining for a table. | Implement "Order" datatype to store and track ordered items by table. And service endpoints to add orders. |
| The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook. | Add these properties to the "Order" datatype. |
| The application MUST, upon deletion request, remove a specified item for a specified table number. | Add service endpoints to remove "Orders". |
| The application MUST, upon query request, show all items for a specified table number. | Add service endpoints to query "Orders" with table id as search option. |
| The application MUST, upon query request, show a specified item for a specified table number. | Implement "Item" datatype with "id", "name", "description". Add service endpoints to query "Item".|
| The application MUST accept at least 10 simultaneous incoming add/remove/query requests. | Actix web can handle by default max connections up to 256 with a thread count of 512 divided by std::thread::available_parallelism(). |
| The client MAY limit the number of specific tables in its requests to a finite set (at least 100). | |
| The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes. |  |
| The application MAY keep the length of time for the item to prepare static (in other words, the time does not have 
to be counted down in real time, only upon item creation and then removed with the item upon item deletion). ||

## Assumptions
- The service runs in an private network and does not require TLS (HTTPS).

## Setup and Run 

To run the 'cargo sqlx' commands, you need to install the sqlx-cli see https://github.com/launchbadge/sqlx/tree/main/sqlx-cli. 

>If the error/warning "set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache" occured.
> You can do as described or run the application/tests again, then it disappear as well.

### Application

```bash
# Set test database
export DATABASE_URL="sqlite:production.db"

# (Optional if db not exists) create db if not exists
cargo sqlx db create

# (Optional if db not exists) Run sql migrations
cargo sqlx migrate run

# Run application
cargo run
```

### Tests
```bash
# Set test database
export DATABASE_URL="sqlite:test.db"

# (Optional if db not exists) create db
cargo sqlx db create

# (Optional if db not exists) Run sql migrations
cargo sqlx migrate run

# Run tests single threaded to reuse the test database for all tests
cargo test -- --test-threads=1
```

# Use Client

You can use any openapi client as you whish with the schema "http://localhost:8080/docs/openapi.json".
Or you open the url http://localhost:8080/docs with a browser, there you get a selfhosted openapi client. 