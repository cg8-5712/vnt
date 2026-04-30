use crate::protocol::transmission::TransmissionBytes;
use crate::tls::verifier::CertValidationMode;
use crate::tunnel_core::server::transport::config::ConnectConfig;
use anyhow::{Context, bail};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use quinn::{ClientConfig, Endpoint, RecvStream, SendStream};
use std::sync::Arc;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[cfg(target_os = "android")]
use crate::socket_protect;
#[cfg(target_os = "android")]
use quinn::{EndpointConfig, default_runtime};

#[derive(Default)]
pub struct QuicTransport {
    framed: Option<(
        FramedWrite<SendStream, LengthDelimitedCodec>,
        FramedRead<RecvStream, LengthDelimitedCodec>,
    )>,
}
impl QuicTransport {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn disconnect(&mut self) {
        self.framed = None;
    }
    pub async fn connect(&mut self, config: &ConnectConfig) -> anyhow::Result<()> {
        if self.framed.is_some() {
            bail!("Already connected");
        }
        let (w, r) = connect_quic(config).await?;
        self.framed = Some((w, r));
        Ok(())
    }
    pub async fn send(&mut self, buf: Bytes) -> anyhow::Result<()> {
        let Some((w, _r)) = self.framed.as_mut() else {
            bail!("Not connected");
        };
        w.send(buf).await.context("send to server failed")
    }
    pub async fn next(&mut self) -> anyhow::Result<TransmissionBytes> {
        let Some((_w, r)) = self.framed.as_mut() else {
            bail!("Not connected");
        };
        r.next()
            .await
            .context("EOF")?
            .context("receive from server failed")
            .map(TransmissionBytes::from)
    }
}
pub async fn connect_quic(
    config: &ConnectConfig,
) -> anyhow::Result<(
    FramedWrite<SendStream, LengthDelimitedCodec>,
    FramedRead<RecvStream, LengthDelimitedCodec>,
)> {
    let server_addr = config.server_addr();
    let server_name = config.server_name();
    let quic_config = create_client_config(&config.cert_mode)?;
    let mut endpoint = match create_client_endpoint((std::net::Ipv6Addr::UNSPECIFIED, 0).into()) {
        Ok(endpoint) => endpoint,
        Err(e) => {
            log::warn!("Failed to create QUIC endpoint: {}", e);
            create_client_endpoint((std::net::Ipv4Addr::UNSPECIFIED, 0).into())
                .context("Failed to create QUIC endpoint")?
        }
    };

    endpoint.set_default_client_config(quic_config);
    let connection = endpoint
        .connect(server_addr, server_name)?
        .await
        .context("Failed to establish QUIC connection")?;
    let (send_stream, recv_stream) = connection
        .open_bi()
        .await
        .context("Failed to open bidirectional stream")?;
    let framed_write = FramedWrite::new(send_stream, LengthDelimitedCodec::new());
    let framed_read = FramedRead::new(recv_stream, LengthDelimitedCodec::new());
    Ok((framed_write, framed_read))
}

fn create_client_config(cert_mode: &CertValidationMode) -> anyhow::Result<ClientConfig> {
    let config = cert_mode.create_tls_client_config()?;
    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(config)
            .context("Failed to create QUIC client config")?,
    ));

    Ok(client_config)
}

fn create_client_endpoint(local_addr: std::net::SocketAddr) -> std::io::Result<Endpoint> {
    #[cfg(target_os = "android")]
    {
        let runtime =
            default_runtime().ok_or_else(|| std::io::Error::other("no async runtime found"))?;
        let socket = socket_protect::bind_std_udp_socket(local_addr, local_addr.is_ipv6())?;
        return Endpoint::new(EndpointConfig::default(), None, socket, runtime);
    }

    #[cfg(not(target_os = "android"))]
    {
        Endpoint::client(local_addr)
    }
}
