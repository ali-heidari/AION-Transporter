use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4}
};
use tokio::net::UdpSocket;

pub async fn listen<F>(on_data_received: F) -> std::io::Result<()>
where
    F: AsyncFn(SocketAddr, &[u8]),
{
    // Bind to port 9999 on all interfaces
    let socket = UdpSocket::bind("0.0.0.0:9999").await?;
    println!("Listening for multicast on 239.255.255.250:9999...");

    // Join the multicast group
    let multicast_addr = Ipv4Addr::new(239, 255, 255, 250);
    socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = vec![0u8; 4096];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];

        on_data_received(src, data).await;
    }
}

pub async fn send(message: &str) -> std::io::Result<()> {
    // Multicast address and port
    let multicast_addr = Ipv4Addr::new(239, 255, 255, 250);
    let port = 9999;

    // Bind to any local interface
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Message to broadcast
    let message = message.as_bytes();

    // Destination socket
    let dest = SocketAddrV4::new(multicast_addr, port);

    socket.send_to(message, dest).await?;

    Ok(())
}
