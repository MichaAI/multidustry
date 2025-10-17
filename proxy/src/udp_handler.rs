use std::io::Cursor;

use binrw::BinWrite;
use bytes::Buf;
use bytes::{BufMut, Bytes, BytesMut};
use kanal::*;
use multidustry_kv::get_storage_instance;
use multidustrycore::kv::{self, get_string_from_db};
use tokio::{self, net::UdpSocket, sync::OnceCell};
use tracing::{debug, info};

use crate::protocol::ping_resp::DiscoveryResponce;

#[derive(Debug)]
pub struct UdpPacket {
    data: Vec<u8>,
    addr: std::net::SocketAddr,
}

static SOCKET: OnceCell<UdpSocket> = OnceCell::const_new();
async fn get_udp_socket() -> &'static UdpSocket {
    SOCKET
        .get_or_init(|| async { UdpSocket::bind("0.0.0.0:6567").await.unwrap() })
        .await
}
pub async fn handle_udp_conections() {
    let cpu_count = num_cpus::get();
    let (tx, rx) = bounded_async(100);

    for _ in 0..cpu_count {
        let rx = rx.clone();
        tokio::spawn(async move {
            worker_udp(rx).await;
        });
    }

    tokio::spawn(async move {
        master_udp(tx).await;
    });
}

async fn master_udp(sender: AsyncSender<UdpPacket>) {
    let socket = get_udp_socket().await;
    loop {
        let mut buf = vec![0u8; 1024];
        match socket.recv_from(&mut buf).await {
            Ok((_, addr)) => {
                let _ = sender
                    .send(UdpPacket {
                        data: buf,
                        addr: addr,
                    })
                    .await;
            }
            Err(e) => {
                eprintln!("Failed to recive from socket: {:?}", e);
            }
        }
    }
}

async fn worker_udp(receiver: AsyncReceiver<UdpPacket>) {
    while let Ok(packet) = receiver.recv().await {
        process_udp_packet(packet).await;
    }
}

async fn process_udp_packet(packet: UdpPacket) {
    let socket = get_udp_socket().await;
    let mut data = &packet.data[..];
    if data.len() < 1 {
        return;
    }
    let first_byte = data.get_i8();
    if first_byte == -2 {
        let db = get_storage_instance().await;
        let custom_gamemode = get_string_from_db(db, "config/custom_gamemode").await;
        info!("Custom gamemode: {}", custom_gamemode);
        let resp = DiscoveryResponce::builder()
            .server_name(get_string_from_db(db, "config/server_name").await.into())
            .map_name(
                get_string_from_db(db, "config/default_world_map_name")
                    .await
                    .into(),
            )
            .total_players(
                get_string_from_db(db, "stats/total_players")
                    .await
                    .parse()
                    .unwrap_or_default(),
            )
            .wave(0)
            .version_type(get_string_from_db(db, "config/version_type").await.into())
            .gamemode(0)
            .description(get_string_from_db(db, "config/description").await.into())
            .custom_gamemode(
                get_string_from_db(db, "config/custom_gamemode")
                    .await
                    .into(),
            )
            .build();

        let mut output = Cursor::new(Vec::new());
        resp.write(&mut output).unwrap();
        socket
            .send_to(&output.into_inner(), packet.addr)
            .await
            .unwrap();
    };
}
