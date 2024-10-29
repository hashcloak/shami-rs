use crate::net::Packet;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    time::{Duration, Instant},
};
use thiserror::Error;

/// Possible errors that may appear in a channel.
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("connection timeout")]
    Timeout,

    #[error("the channel is not alive")]
    NotAlive,

    #[error("channel buffer is empty")]
    EmptyBuffer,
}

/// Defines a channel of the network.
pub trait Channel {
    /// Closes a channel.
    fn close(&mut self) -> anyhow::Result<()>;
    /// Send a packet using the current channel.
    fn send(&mut self, packet: &Packet) -> anyhow::Result<usize>;
    /// Receives a packet from the current channel.
    fn recv(&mut self) -> anyhow::Result<Packet>;
}

/// Representation of a TCP channel between two parties.
#[derive(Debug, Default)]
pub struct TcpChannel {
    /// Stream for the channel. If the channel is not connected
    /// then the stream will be `None`.
    stream: Option<TcpStream>,
}

impl TcpChannel {
    /// Accepts a connection in the corresponding listener.
    pub(crate) fn accept_connection(listener: &TcpListener) -> anyhow::Result<(TcpChannel, usize)> {
        let (mut channel, socket) = listener.accept()?;

        // Once the client is connected, we receive his ID from the current established channel.
        let mut id_buffer = [0; (usize::BITS / 8) as usize];
        channel.read_exact(&mut id_buffer)?;
        let remote_id = usize::from_le_bytes(id_buffer);
        log::info!(
            "accepted connection request acting like a server from {:?} with ID {}",
            socket,
            remote_id,
        );

        Ok((
            Self {
                stream: Some(channel),
            },
            remote_id,
        ))
    }

    /// Connect to the remote address as a client using the corresponding timeout. The party
    /// tries to connect to the "server" multiple times using a sleep time between calls.
    /// If the "server" party does not answer within the timeout, then the function returns
    /// an error.
    pub(crate) fn connect_as_client(
        local_id: usize,
        remote_addr: SocketAddr,
        timeout: Duration,
        sleep_time: Duration,
    ) -> anyhow::Result<TcpChannel> {
        let start_time = Instant::now();

        // Repeatedly tries to connect to the server during the timeout.
        log::info!("trying to connect as a client to {:?}", remote_addr);
        loop {
            match TcpStream::connect(remote_addr) {
                Ok(mut stream) => {
                    // Send the id of the party that is connecting to the
                    // server once the connection is successfull.
                    stream.write_all(&local_id.to_le_bytes())?;

                    log::info!(
                        "connected successfully with {:?} using the local port {:?}",
                        remote_addr,
                        stream.local_addr()?
                    );

                    return Ok(Self {
                        stream: Some(stream),
                    });
                }
                Err(_) => {
                    let elapsed = start_time.elapsed();
                    if elapsed > timeout {
                        // At this moment the enlapsed time passed the timeout. Hence we return an
                        // error. Tired of waiting for the "server" to be ready.
                        log::error!(
                            "timeout reached, server not listening from ID {local_id} to server {:?}",
                            remote_addr
                        );
                        anyhow::bail!(ChannelError::Timeout)
                    }
                    // The connection was not successfull. Hence, we try to connect again with the
                    // "server" party.
                    std::thread::sleep(sleep_time)
                }
            }
        }
    }
}

impl Channel for TcpChannel {
    fn close(&mut self) -> anyhow::Result<()> {
        if let Some(stream) = &self.stream {
            stream.shutdown(std::net::Shutdown::Both)?;
        }
        self.stream = None;
        log::info!("channel successfully closed");
        Ok(())
    }

    fn send(&mut self, packet: &Packet) -> anyhow::Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                // First, we need to send the size of the packet to be able to know the amout
                // of bits that are being sent.
                let packet_size = packet.size();
                let bytes_size_packet = bincode::serialize(&packet_size)?;
                stream.write_all(&bytes_size_packet)?;

                // Then, we send the actual packet.
                stream.write_all(packet.as_slice())?;
                log::info!(
                    "sent packet to peer {:?} with {} bytes",
                    stream.peer_addr()?,
                    packet.size()
                );
                Ok(packet.size())
            }
            None => {
                log::error!("the channel is not connected yet");
                anyhow::bail!(ChannelError::NotAlive)
            }
        }
    }

    fn recv(&mut self) -> anyhow::Result<Packet> {
        match &mut self.stream {
            Some(stream) => {
                let mut buffer_packet_size = [0; (usize::BITS / 8) as usize];
                stream.read_exact(&mut buffer_packet_size)?;
                let packet_size: usize = bincode::deserialize(&buffer_packet_size)?;

                // Then, we receive the buffer the amount bytes until the end is reached.
                let mut payload_buffer = vec![0; packet_size];
                stream.read_exact(&mut payload_buffer)?;

                log::info!(
                    "received packet from peer {:?} with {} bytes",
                    stream.peer_addr(),
                    packet_size,
                );

                Ok(Packet::new(payload_buffer))
            }
            None => {
                log::error!("the channel is not alive to receive information");
                anyhow::bail!(ChannelError::NotAlive)
            }
        }
    }
}

/// This is a channel used when a party wants to connect with himself.
#[derive(Default)]
pub struct LoopBackChannel {
    /// Queue of incomming channels.
    buffer: VecDeque<Packet>,
}

impl Channel for LoopBackChannel {
    fn close(&mut self) -> anyhow::Result<()> {
        self.buffer.clear();
        log::info!("channel successfully closed");
        Ok(())
    }

    fn send(&mut self, packet: &Packet) -> anyhow::Result<usize> {
        log::info!("sent {} bytes to myself", packet.0.len());
        self.buffer.push_back(Packet::from(packet.as_slice()));
        Ok(packet.0.len())
    }

    fn recv(&mut self) -> anyhow::Result<Packet> {
        log::info!("received packet from myself");
        self.buffer
            .pop_front()
            .ok_or(anyhow::Error::new(ChannelError::EmptyBuffer))
    }
}
