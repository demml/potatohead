pub mod health;
pub mod openai;
pub mod server;
use potato_error::PotatoError;
pub use server::create_app;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::info;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with routes
    let app = create_app()
        .await
        .map_err(|e| PotatoError::Error(e.to_string()))?;
    let addr = format!("0.0.0.0:{}", "3000");

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    println!("🚀 Server Running 🚀");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

pub fn start_server_in_background() -> Arc<Mutex<Option<JoinHandle<()>>>> {
    let handle = Arc::new(Mutex::new(None));
    let handle_clone = handle.clone();

    tokio::spawn(async move {
        let server_handle = tokio::spawn(async {
            if let Err(e) = start_server().await {
                eprintln!("Server error: {}", e);
            }
        });

        *handle_clone.lock().await = Some(server_handle);
    });

    handle
}

pub async fn stop_server(handle: Arc<Mutex<Option<JoinHandle<()>>>>) {
    if let Some(handle) = handle.lock().await.take() {
        handle.abort();
        info!("Server stopped");
    }
}
