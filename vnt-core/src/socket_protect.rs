use parking_lot::Mutex;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, OnceLock};
use tokio::net::{TcpSocket, TcpStream, ToSocketAddrs, UdpSocket};

type SocketProtector = Arc<dyn Fn(i32) -> anyhow::Result<()> + Send + Sync + 'static>;

static SOCKET_PROTECTOR: OnceLock<Mutex<Option<SocketProtector>>> = OnceLock::new();

fn socket_protector() -> &'static Mutex<Option<SocketProtector>> {
    SOCKET_PROTECTOR.get_or_init(|| Mutex::new(None))
}

pub fn set_socket_protector(protector: Option<SocketProtector>) {
    *socket_protector().lock() = protector;
}

#[cfg(target_os = "android")]
fn protect_socket_fd(fd: i32) -> anyhow::Result<()> {
    use anyhow::Context;

    let protector = socket_protector().lock().clone();
    if let Some(protector) = protector {
        protector(fd).with_context(|| format!("VpnService.protect({fd}) failed"))?;
    }
    Ok(())
}

#[cfg(target_os = "android")]
pub fn protect_tcp_socket(socket: &TcpSocket) -> anyhow::Result<()> {
    use std::os::fd::AsRawFd;

    protect_socket_fd(socket.as_raw_fd())
}

#[cfg(not(target_os = "android"))]
pub fn protect_tcp_socket(_socket: &TcpSocket) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn protect_udp_socket(socket: &UdpSocket) -> anyhow::Result<()> {
    use std::os::fd::AsRawFd;

    protect_socket_fd(socket.as_raw_fd())
}

#[cfg(not(target_os = "android"))]
pub fn protect_udp_socket(_socket: &UdpSocket) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn protect_std_udp_socket(socket: &std::net::UdpSocket) -> anyhow::Result<()> {
    use std::os::fd::AsRawFd;

    protect_socket_fd(socket.as_raw_fd())
}

#[cfg(not(target_os = "android"))]
pub fn protect_std_udp_socket(_socket: &std::net::UdpSocket) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "android")]
pub fn protect_socket2<S>(socket: &S) -> anyhow::Result<()>
where
    S: std::os::fd::AsRawFd,
{
    use std::os::fd::AsRawFd;

    protect_socket_fd(socket.as_raw_fd())
}

#[cfg(not(target_os = "android"))]
pub fn protect_socket2<S>(_socket: &S) -> anyhow::Result<()> {
    Ok(())
}

pub async fn connect_tcp<A>(addr: A) -> io::Result<TcpStream>
where
    A: ToSocketAddrs,
{
    let mut last_error = None;
    for socket_addr in tokio::net::lookup_host(addr).await? {
        match connect_tcp_addr(socket_addr).await {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "no TCP target address resolved",
        )
    }))
}

async fn connect_tcp_addr(addr: SocketAddr) -> io::Result<TcpStream> {
    #[cfg(target_os = "android")]
    {
        let socket = if addr.is_ipv4() {
            TcpSocket::new_v4()?
        } else {
            TcpSocket::new_v6()?
        };
        socket.set_nodelay(true)?;
        protect_tcp_socket(&socket).map_err(io::Error::other)?;
        return socket.connect(addr).await;
    }

    #[cfg(not(target_os = "android"))]
    {
        let stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        Ok(stream)
    }
}

pub async fn bind_udp_any(remote_addr: SocketAddr) -> io::Result<UdpSocket> {
    let bind_addr = if remote_addr.is_ipv4() {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    };
    let socket = UdpSocket::bind(bind_addr).await?;
    protect_udp_socket(&socket).map_err(io::Error::other)?;
    Ok(socket)
}

pub fn bind_std_udp_socket(
    local_addr: SocketAddr,
    allow_dual_stack: bool,
) -> io::Result<std::net::UdpSocket> {
    let socket = if local_addr.is_ipv4() {
        socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?
    } else {
        let socket = socket2::Socket::new(
            socket2::Domain::IPV6,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;
        if allow_dual_stack {
            if let Err(error) = socket.set_only_v6(false) {
                log::debug!("unable to make UDP socket dual-stack: {}", error);
            }
        } else {
            socket.set_only_v6(true)?;
        }
        socket
    };
    protect_socket2(&socket).map_err(io::Error::other)?;
    socket.bind(&local_addr.into())?;
    socket.set_nonblocking(true)?;
    Ok(socket.into())
}
