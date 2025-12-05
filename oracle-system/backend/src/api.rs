use warp::Filter;
use std::sync::Arc;
use crate::database::Database;

pub async fn start_api_server(database: Arc<Database>) {
    let db_filter = warp::any().map(move || database.clone());

    let price_route = warp::path!("oracle" / "price" / String)
        .and(db_filter.clone())
        .and_then(handle_get_price);

    let health_route = warp::path!("oracle" / "health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    let routes = price_route.or(health_route);

    println!("API Server starting on port 3030...");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn handle_get_price(symbol: String, db: Arc<Database>) -> Result<impl warp::Reply, warp::Rejection> {
    match db.get_cached_price(&symbol).await {
        Ok(Some(price)) => Ok(warp::reply::json(&price)),
        Ok(None) => Err(warp::reject::not_found()),
        Err(_) => Err(warp::reject::not_found()), // Simplify error handling for demo
    }
}
