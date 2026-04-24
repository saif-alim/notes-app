use std::sync::Arc;

use notes_api::{create_router, InMemoryNotesStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("notes-api listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
