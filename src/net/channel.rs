use crate::net::Packet;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    time::{Duration, Instant},
};
use thiserror::Error;

/// Possible errors that may appear in a channel.
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("channel reception error")]
    ChannelRecvError,

    #[error("channel sending error")]
    ChannelSendError,

    #[error("channel is not alive")]
    ChannelNotAlive,

    #[error("the channel is empty")]
    EmptyBufferError,

    #[error("can not retrieve the peer")]
    PeerUnknown,

    #[error("the channel has not received any data")]
    NoData,
}

/// Defines a channel of the network.
pub trait Channel {
    /// Closes a channel.
    fn close(&mut self) -> io::Result<()>;
    /// Send a packet using the current channel.
    fn send(&mut self, packet: &Packet) -> Result<usize, ChannelError>;
    /// Receives a packet from the current channel.
    fn recv(&mut self) -> Result<Packet, ChannelError>;
}

/// Representation of a TCP channel between two parties.
#[derive(Debug, Default)]
pub struct TcpChannel {
    /// Stream for the channel. If the channel is not connected
    /// then the stream will be `None`.
    stream: Option<TcpStream>,
}

impl TcpChannel {
    /// Maximum size of the buffer that is received using the TCP channel.
    const MAX_BUFFER_SIZE: usize = 1024;
    /// Maximum default timeout for reception in ms.
    const DEFAULT_RECV_TIMEOUT: u64 = 10000;

    /// Accepts a connection in the corresponding listener.
    pub(crate) fn accept_connection(listener: &TcpListener) -> io::Result<(TcpChannel, usize)> {
        let (mut channel, socket) = listener.accept()?;

        // Once the client is connected, we receive his ID from the current established channel.
        let mut id_buffer = [0; (usize::BITS / 8) as usize];
        channel.read_exact(&mut id_buffer).map_err(|err| {
            log::error!("error while reading the ID of the peer {:?}", err);
            err
        })?;
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
    ) -> io::Result<TcpChannel> {
        let start_time = Instant::now();

        // Repeatedly tries to connect to the server during the timeout.
        log::info!("trying to connect as a client to {:?}", remote_addr);
        loop {
            match TcpStream::connect(remote_addr) {
                Ok(mut stream) => {
                    // Send the id of the party that is connecting to the
                    // server once the connection is successfull.
                    stream
                        .write_all(&local_id.to_le_bytes())
                        .inspect_err(|err| {
                            log::error!(
                            "error connecting as a client while sending the local ID {} to {:?}: {err}",
                            local_id,
                            stream.peer_addr()
                        );
                        })?;

                    log::info!(
                        "connected successfully with {:?} using the local port {:?}",
                        remote_addr,
                        stream.local_addr()?
                    );
                    return Ok(Self {
                        stream: Some(stream),
                    });
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    if elapsed > timeout {
                        // At this moment the enlapsed time passed the timeout. Hence we return an
                        // error. Tired of waiting for the "server" to be ready.
                        log::error!(
                            "timeout reached, server not listening from ID {local_id} to server {:?}",
                            remote_addr
                        );
                        return Err(io::Error::from(e.kind()));
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
    fn close(&mut self) -> io::Result<()> {
        if let Some(stream) = &self.stream {
            stream.shutdown(std::net::Shutdown::Both)?;
        }
        self.stream = None;
        log::info!("channel successfully created");
        Ok(())
    }

    fn send(&mut self, packet: &Packet) -> Result<usize, ChannelError> {
        match &mut self.stream {
            Some(stream) => {
                let bytes = stream.write(packet.as_slice()).map_err(|err| {
                    log::error!(
                        "error writing the packet to {:?}: {:?}",
                        stream.peer_addr(),
                        err,
                    );
                    ChannelError::ChannelSendError
                })?;
                log::info!(
                    "sent packet to peer {:?} with {} bytes",
                    stream.peer_addr().map_err(|err| {
                        log::error!("Peer unknown: {:?}", err);
                        ChannelError::PeerUnknown
                    })?,
                    bytes
                );
                Ok(bytes)
            }
            None => {
                log::error!("the channel is not connected yet");
                Err(ChannelError::ChannelNotAlive)
            }
        }
    }

    fn recv(&mut self) -> Result<Packet, ChannelError> {
        match &mut self.stream {
            Some(stream) => {
                let initial_time = Instant::now();
                // Blocks the channel for a certain timeout until it receives the message. If the
                // timeout is completed, then an error is emmited.
                let (buffer, bytes) = loop {
                    let mut buffer = [0; Self::MAX_BUFFER_SIZE];
                    let bytes = stream.read(&mut buffer).map_err(|err| {
                        log::error!(
                            "error receiving packet from peer {:?}: {:?}",
                            stream.peer_addr(),
                            err
                        );
                        ChannelError::ChannelRecvError
                    })?;

                    if bytes == 0 {
                        let elapsed_time = Instant::now() - initial_time;
                        if elapsed_time > Duration::from_millis(Self::DEFAULT_RECV_TIMEOUT) {
                            return Err(ChannelError::NoData);
                        }
                    } else {
                        break (buffer, bytes);
                    }
                };
                log::info!(
                    "received packet from peer {:?} with {} bytes",
                    stream.peer_addr(),
                    bytes
                );
                Ok(Packet::from(&buffer[0..bytes]))
            }
            None => {
                log::error!("the channel is not alive to receive information");
                Err(ChannelError::ChannelNotAlive)
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
    fn close(&mut self) -> io::Result<()> {
        self.buffer.clear();
        Ok(())
    }

    fn send(&mut self, packet: &Packet) -> Result<usize, ChannelError> {
        log::info!("sent {} bytes to myself", packet.0.len());
        self.buffer.push_back(Packet::from(packet.as_slice()));
        Ok(packet.0.len())
    }

    fn recv(&mut self) -> Result<Packet, ChannelError> {
        log::info!("received packet from myself");
        self.buffer
            .pop_front()
            .ok_or(ChannelError::EmptyBufferError)
    }
}
