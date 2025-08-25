use bytes::Buf;
use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
    self,
    net::UdpSocket,
    sync::{
        OnceCell,
        mpsc::{Receiver, Sender},
    },
};

#[derive(Debug)]
pub struct UdpPacket {
    data: Vec<u8>,
    addr: std::net::SocketAddr,
}

static SOCKET: OnceCell<UdpSocket> = OnceCell::const_new();
async fn get_socket() -> &'static UdpSocket {
    SOCKET
        .get_or_init(|| async { UdpSocket::bind("0.0.0.0:6567").await.unwrap() })
        .await
}
pub async fn handle_udp_conections() {
    let cpu_count = num_cpus::get();
    let mut worker_senders = Vec::new();

    for _ in 0..cpu_count {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<UdpPacket>(100);
        worker_senders.push(tx);
        tokio::spawn(async move {
            worker_udp(rx).await;
        });
    }

    tokio::spawn(async move {
        master_udp(worker_senders).await;
    });
}

async fn master_udp(senders: Vec<Sender<UdpPacket>>) {
    let socket = get_socket().await;
    let mut rr_index = 0;
    loop {
        let mut buf = vec![0u8; 1024];
        match socket.recv_from(&mut buf).await {
            Ok((_, addr)) => {
                let sender = &senders[rr_index];
                let _ = sender
                    .send(UdpPacket {
                        data: buf,
                        addr: addr,
                    })
                    .await;
                rr_index = (rr_index + 1) % senders.len();
            }
            Err(_) => {}
        }
    }
}

async fn worker_udp(mut receiver: Receiver<UdpPacket>) {
    while let Some(packet) = receiver.recv().await {
        process_udp_packet(packet).await;
    }
}

async fn process_udp_packet(packet: UdpPacket) {
    let socket = get_socket().await;
    let mut data = &packet.data[..];
    if data.len() < 1 {
        return;
    }
    let first_byte = data.get_i8();
    if first_byte == -2 {
        let mut buf = BytesMut::new();
        let response = constuct_discovery_responce(&mut buf);
        if let Err(e) = socket.send_to(&response[..], packet.addr).await {
            eprintln!("Ошибка отправки UDP: {e}");
        }
    };
}

fn constuct_discovery_responce(buf: &mut BytesMut) -> Bytes {
    write_string(buf, "Neodustry", 100); // Server name
    write_string(buf, "HUB", 100); // Map name
    buf.put_i32(0); // Total players
    buf.put_i32(0); // Wave
    buf.put_i32(146); // Version number
    write_string(buf, "multidustry", 64); // Version type
    buf.put_i8(1); // Gamemode
    buf.put_i32(255); // Player limit
    write_string(buf, "Test multidustry desc", 100); // Description
    write_string(buf, "HUB", 64); // Custom gamemode
    buf.put_i16(6567); // Server port

    buf.clone().freeze()
}

pub fn write_string(buf: &mut BytesMut, s: &str, max_length: usize) {
    // UTF-8 байты строки
    let bytes = s.as_bytes();

    // Обрезаем по лимиту байтов
    let trimmed_len = bytes.len().min(max_length);

    // ВАЖНО: длина в одном байте (u8), как в оригинале
    // Если trimmed_len > 255, по протоколу длина уходит в один байт,
    // значит надо дополнительно ограничить 255.
    let len_u8 = trimmed_len.min(u8::MAX as usize) as u8;

    // Резервируем место: 1 байт под длину + сами данные
    buf.reserve(1 + len_u8 as usize);

    // Пишем длину (1 байт)
    buf.put_u8(len_u8);

    // Пишем байты
    buf.put_slice(&bytes[..len_u8 as usize]);
}
