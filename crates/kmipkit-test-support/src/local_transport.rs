//! Loopback-only socket fixtures for transport tests.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};

/// A TCP listener bound to an ephemeral IPv4 loopback port.
pub struct LoopbackTcpListener {
    listener: TcpListener,
    local_addr: SocketAddr,
}

impl LoopbackTcpListener {
    /// Binds a listener to `127.0.0.1` on an operating-system-selected port.
    ///
    /// # Errors
    ///
    /// Returns an error if the loopback address cannot be bound or queried.
    pub fn bind() -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let local_addr = listener.local_addr()?;
        Ok(Self {
            listener,
            local_addr,
        })
    }

    /// Returns the ephemeral loopback address.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Transfers the bound listener to a test peer.
    #[must_use]
    pub fn into_inner(self) -> TcpListener {
        self.listener
    }
}
