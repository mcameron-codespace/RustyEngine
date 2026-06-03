pub mod protocol;
pub mod rollback;

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};

pub use protocol::{PlayerInputs, ClientMessage, ServerMessage, NetworkChannel};
pub use rollback::{PredictionBuffer, PredictedFrame};

// Mock Renet configurations to preserve compiler dependency contracts 
// inside a self-contained compilation sandbox
pub struct NetworkConfig {
    pub server_addr: SocketAddr,
    pub max_clients: usize,
}

pub struct NetworkSystem {
    pub socket: UdpSocket,
    pub is_server: bool,
    pub incoming_packets: VecDeque<Vec<u8>>,
}

impl NetworkSystem {
    /// Bind a raw UDP transport socket configured for either peer client or server loopbacks
    pub fn new(bind_addr: &str, is_server: bool) -> Self {
        let socket = UdpSocket::bind(bind_addr).expect("Failed to bind UDP port interface");
        socket.set_nonblocking(true).expect("Failed to enable non-blocking UDP mechanics");

        Self {
            socket,
            is_server,
            incoming_packets: VecDeque::with_capacity(256),
        }
    }

    /// Check connection events and buffer raw network payloads (Phase 5.1 Verification)
    pub fn poll_sockets(&mut self) {
        let mut buffer = [0u8; 1500]; // Standard network MTU size limit
        while let Ok((bytes_read, _source_addr)) = self.socket.recv_from(&mut buffer) {
            let payload = buffer[..bytes_read].to_vec();
            self.incoming_packets.push_back(payload);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_buffer_cycling() {
        let mut buffer = PredictionBuffer::with_capacity(60); // 60-frame rollback window

        let initial_inputs = PlayerInputs {
            forward: true,
            backward: false,
            left: false,
            right: false,
            yaw: 0.0,
            pitch: 0.0,
        };

        // Populate a predicted position at tick 10
        buffer.insert(10, initial_inputs, [5.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        // Assert we retrieve the matching data perfectly
        let frame = buffer.get(10).expect("Tick 10 should exist");
        assert_eq!(frame.position, [5.0, 0.0, 0.0]);
        assert!(frame.inputs.forward);

        // Verify that buffer indices wrap and cycle without error
        buffer.insert(70, initial_inputs, [15.0, 0.0, 0.0], [1.0, 0.0, 0.0]); // 70 % 60 = 10 (overwrites)
        let overwritten_frame = buffer.get(10);
        // Stale tick 10 gets evicted in favor of tick 70
        assert_eq!(overwritten_frame.unwrap().tick, 70);
    }

    #[test]
    fn test_rollback_desync_trigger() {
        let mut buffer = PredictionBuffer::with_capacity(120);
        let inputs = PlayerInputs::empty();

        // Client predicts player is at [10.0, 0.0, 0.0] on tick 45
        buffer.insert(45, inputs, [10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

        // 1. Server returns snapshot within the correction threshold (Tolerance = 5cm / 0.05m)
        let tiny_deviation = [10.02, 0.0, 0.0];
        let rollback_not_needed = buffer.check_desync(45, tiny_deviation, 0.05).unwrap();
        assert!(!rollback_not_needed);

        // 2. Server returns massive deviation (representing latency rubber-banding or external impact forces)
        let large_deviation = [12.0, 0.0, 0.0];
        let rollback_needed = buffer.check_desync(45, large_deviation, 0.05).unwrap();
        assert!(rollback_needed);
    }
}