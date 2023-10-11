pub mod queue;
pub mod router;
pub mod state;

use self::state::AppState;
use hextacy::env;
use tracing::info;

pub async fn start_server(state: AppState) -> Result<(), std::io::Error> {
    let (host, port) = (
        env::get_or_default("HOST", "127.0.0.1"),
        env::get_or_default("PORT", "3000"),
    );

    let addr = format!("{host}:{port}");

    info!("Starting server on {addr}");

    let router = router::router(&state).await;

    axum::Server::bind(&addr.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .expect("couldn't start server");

    Ok(())
}
