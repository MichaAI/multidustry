use tokio::{net::TcpListener, sync::OnceCell};

static SOCKET: OnceCell<TcpListener> = OnceCell::const_new();
pub async fn get_tcp_socket() -> &'static TcpListener {
    SOCKET
        .get_or_init(|| async { TcpListener::bind("0.0.0.0:6567").await.unwrap() })
        .await
}

pub async fn handle_tcp() {
    let socket = get_tcp_socket().await;
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
    }
}
