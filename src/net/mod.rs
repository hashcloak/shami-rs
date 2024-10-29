pub mod channel;

use crate::net::channel::{Channel, TcpChannel};
use channel::LoopBackChannel;
use std::{
    cmp::Ordering,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    time::Duration,
};

/// Packet of information sent through a given channel.
pub struct Packet(Vec<u8>);

impl Packet {
    /// Creates a new packet.
    pub fn new(buffer: Vec<u8>) -> Self {
        Self(buffer)
    }

    /// Returns an slice to the packet.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns the size of the packet.
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl From<&[u8]> for Packet {
    fn from(value: &[u8]) -> Self {
        Self(Vec::from(value))
    }
}

/// Network that contains all the channels connected to the party. Each channel is
/// a connection to other parties.
pub struct Network {
    /// Channnels for each peer.
    peer_channels: Vec<Box<dyn Channel>>,
}

impl Network {
    /// Base port used to find the port to the corresponding party. The port `BASE_PORT + i` is
    /// assigned to the party i.
    pub const BASE_PORT: u16 = 5000;

    /// IP of the localhost.
    pub const LOCALHOST_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

    /// Timeout to wait for connections.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(100);

    /// Default time to sleep between conetion trials.
    pub const DEFAULT_SLEEP: Duration = Duration::from_millis(500);

    /// Creates a new network using the ID of the current party and the number of parties connected
    /// to the network.
    pub fn create(id: usize, n_parties: usize) -> anyhow::Result<Self> {
        log::info!("creating network");
        let server_port = Self::BASE_PORT + id as u16;
        let server_address = SocketAddr::new(std::net::IpAddr::V4(Self::LOCALHOST_IP), server_port);
        let server_listener = TcpListener::bind(server_address)?;
        log::info!("listening on {:?}", server_address);

        let mut peers: Vec<Box<dyn Channel>> = Vec::new();
        for i in 0..n_parties {
            if i != id {
                peers.push(Box::new(TcpChannel::default()));
            } else {
                peers.push(Box::new(LoopBackChannel::default()));
            }
        }

        for i in 0..n_parties {
            match i.cmp(&id) {
                Ordering::Less => {
                    log::info!("connecting as a client with peer ID {i}");
                    let remote_port = Self::BASE_PORT + i as u16;
                    let remote_address =
                        SocketAddr::new(std::net::IpAddr::V4(Self::LOCALHOST_IP), remote_port);
                    let channel = TcpChannel::connect_as_client(
                        id,
                        remote_address,
                        Self::DEFAULT_TIMEOUT,
                        Self::DEFAULT_SLEEP,
                    )?;
                    peers[i] = Box::new(channel);
                }
                Ordering::Greater => {
                    log::info!("acting as a server for peer ID {i}");
                    let (channel, remote_id) = TcpChannel::accept_connection(&server_listener)?;
                    peers[remote_id] = Box::new(channel);
                }
                Ordering::Equal => {
                    log::info!("adding the loop-back channel");
                    peers[i] = Box::new(LoopBackChannel::default());
                }
            }
        }
        Ok(Self {
            peer_channels: peers,
        })
    }

    /// Send a packet to every party in the network.
    pub fn send(&mut self, packet: &Packet) -> anyhow::Result<usize> {
        let mut bytes_sent = 0;
        for i in 0..self.peer_channels.len() {
            bytes_sent = self
                .peer_channels
                .get_mut(i)
                .expect("channel index not found")
                .send(packet)?;
        }
        Ok(bytes_sent)
    }

    /// Receive a packet from each party in the network.
    pub fn recv(&mut self) -> anyhow::Result<Vec<Packet>> {
        let mut packets = Vec::new();
        for i in 0..self.peer_channels.len() {
            let packet = self
                .peer_channels
                .get_mut(i)
                .expect("channel index not found")
                .recv()?;
            packets.push(packet);
        }

        Ok(packets)
    }

    /// Closes the network by closing each channel.
    pub fn close(&mut self) -> anyhow::Result<()> {
        for i in 0..self.peer_channels.len() {
            self.peer_channels
                .get_mut(i)
                .expect("channel index not found")
                .close()?;
        }
        Ok(())
    }

    /// Sends a packet of information to a given party.
    pub fn send_to(&mut self, packet: &Packet, party_id: usize) -> anyhow::Result<usize> {
        let bytes_sent = self.peer_channels[party_id].send(packet)?;
        Ok(bytes_sent)
    }

    /// Receives a packet from a given party.
    pub fn recv_from(&mut self, party_id: usize) -> anyhow::Result<Packet> {
        let packet = self.peer_channels[party_id].recv()?;
        Ok(packet)
    }
}
