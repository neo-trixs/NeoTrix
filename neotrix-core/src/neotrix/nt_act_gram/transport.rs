use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const ABRIDGED_HEADER: [u8; 1] = [0xef];

pub enum TransportProtocol {
    Abridged,
    Intermediate,
}

pub struct MtpTransport {
    stream: TcpStream,
    protocol: TransportProtocol,
}

impl MtpTransport {
    pub fn new(stream: TcpStream, protocol: TransportProtocol) -> Self {
        Self { stream, protocol }
    }

    pub async fn send_handshake(&mut self) -> Result<(), String> {
        match self.protocol {
            TransportProtocol::Abridged => {
                self.stream.write_all(&ABRIDGED_HEADER).await.map_err(|e| e.to_string())
            }
            TransportProtocol::Intermediate => {
                self.stream.write_all(&[0xee, 0xee, 0xee, 0xee]).await.map_err(|e| e.to_string())
            }
        }
    }

    pub async fn read_handshake(&mut self) -> Result<TransportProtocol, String> {
        let mut first = [0u8; 1];
        self.stream.read_exact(&mut first).await.map_err(|e| format!("read handshake: {}", e))?;
        if first[0] == 0xef {
            self.protocol = TransportProtocol::Abridged;
            Ok(TransportProtocol::Abridged)
        } else if first[0] == 0xee {
            let mut rest = [0u8; 3];
            self.stream.read_exact(&mut rest).await.map_err(|e| format!("read int handshake: {}", e))?;
            if rest == [0xee, 0xee, 0xee] {
                self.protocol = TransportProtocol::Intermediate;
                Ok(TransportProtocol::Intermediate)
            } else {
                Err("invalid intermediate handshake".to_string())
            }
        } else {
            Err(format!("unknown protocol byte: 0x{:02x}", first[0]))
        }
    }

    pub async fn read_packet(&mut self) -> Result<Vec<u8>, String> {
        match self.protocol {
            TransportProtocol::Abridged => self.read_abridged().await,
            TransportProtocol::Intermediate => self.read_intermediate().await,
        }
    }

    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), String> {
        match self.protocol {
            TransportProtocol::Abridged => self.write_abridged(data).await,
            TransportProtocol::Intermediate => self.write_intermediate(data).await,
        }
    }

    async fn read_abridged(&mut self) -> Result<Vec<u8>, String> {
        let mut len_byte = [0u8; 1];
        self.stream.read_exact(&mut len_byte).await.map_err(|e| format!("read len: {}", e))?;
        let packet_len = if len_byte[0] == 0x7f {
            let mut len_bytes = [0u8; 3];
            self.stream.read_exact(&mut len_bytes).await.map_err(|e| format!("read len3: {}", e))?;
            u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], 0]) as usize * 4
        } else {
            len_byte[0] as usize * 4
        };
        let mut data = vec![0u8; packet_len];
        self.stream.read_exact(&mut data).await.map_err(|e| format!("read data: {}", e))?;
        Ok(data)
    }

    async fn write_abridged(&mut self, data: &[u8]) -> Result<(), String> {
        if !data.len().is_multiple_of(4) {
            return Err("data len not aligned to 4".to_string());
        }
        let q = data.len() / 4;
        if q < 0x7f {
            let header = [q as u8];
            self.stream.write_all(&header).await.map_err(|e| e.to_string())?;
        } else {
            let q_bytes = (q as u32).to_le_bytes();
            // abridged: [0x7f, b0, b1, b2] for 24-bit LE length
            let header = [0x7f, q_bytes[0], q_bytes[1], q_bytes[2]];
            self.stream.write_all(&header).await.map_err(|e| e.to_string())?;
        }
        self.stream.write_all(data).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn read_intermediate(&mut self) -> Result<Vec<u8>, String> {
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await.map_err(|e| format!("read len: {}", e))?;
        let packet_len = u32::from_le_bytes(len_bytes) as usize;
        let mut data = vec![0u8; packet_len];
        self.stream.read_exact(&mut data).await.map_err(|e| format!("read data: {}", e))?;
        Ok(data)
    }

    async fn write_intermediate(&mut self, data: &[u8]) -> Result<(), String> {
        let len = (data.len() as u32).to_le_bytes();
        self.stream.write_all(&len).await.map_err(|e| e.to_string())?;
        self.stream.write_all(data).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn close(mut self) -> Result<(), String> {
        self.stream.shutdown().await.map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_abridged_roundtrip() {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("should bind to random port");
        let addr = listener.local_addr().expect("should get local address");
        let server = tokio::spawn(async move {
            let (s, _) = listener.accept().await.expect("server should accept connection");
            let mut t = MtpTransport::new(s, TransportProtocol::Abridged);
            t.read_handshake().await.expect("server should read handshake");
            let p = t.read_packet().await.expect("server should read packet");
            t.write_packet(&p).await.expect("server should write packet");
        });
        let cs = TcpStream::connect(addr).await.expect("client should connect to server");
        let mut t = MtpTransport::new(cs, TransportProtocol::Abridged);
        t.send_handshake().await.expect("client should send handshake");
        let payload = b"\x01\x02\x03\x04";
        t.write_packet(payload).await.expect("client should write packet");
        let resp = t.read_packet().await.expect("client should read packet");
        assert_eq!(resp, payload);
        server.await.expect("server task should complete");
    }

    #[tokio::test]
    async fn test_intermediate_roundtrip() {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("should bind to random port");
        let addr = listener.local_addr().expect("should get local address");
        let server = tokio::spawn(async move {
            let (s, _) = listener.accept().await.expect("server should accept connection");
            let mut t = MtpTransport::new(s, TransportProtocol::Intermediate);
            t.read_handshake().await.expect("server should read handshake");
            let p = t.read_packet().await.expect("server should read packet");
            t.write_packet(&p).await.expect("server should write packet");
        });
        let cs = TcpStream::connect(addr).await.expect("client should connect to server");
        let mut t = MtpTransport::new(cs, TransportProtocol::Intermediate);
        t.send_handshake().await.expect("client should send handshake");
        let payload = b"\x05\x06\x07\x08";
        t.write_packet(payload).await.expect("client should write packet");
        let resp = t.read_packet().await.expect("client should read packet");
        assert_eq!(resp, payload);
        server.await.expect("server task should complete");
    }

    #[tokio::test]
    async fn test_large_abridged() {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("should bind to random port");
        let addr = listener.local_addr().expect("should get local address");
        let server = tokio::spawn(async move {
            let (s, _) = listener.accept().await.expect("server should accept connection");
            let mut t = MtpTransport::new(s, TransportProtocol::Abridged);
            t.read_handshake().await.expect("server should read handshake");
            let p = t.read_packet().await.expect("server should read packet");
            assert_eq!(p.len(), 512);
            t.write_packet(&p).await.expect("server should write packet");
        });
        let cs = TcpStream::connect(addr).await.expect("client should connect to server");
        let mut t = MtpTransport::new(cs, TransportProtocol::Abridged);
        t.send_handshake().await.expect("client should send handshake");
        let payload: Vec<u8> = (0..512).map(|i| i as u8).collect();
        t.write_packet(&payload).await.expect("client should write packet");
        let resp = t.read_packet().await.expect("client should read packet");
        assert_eq!(resp, payload);
        server.await.expect("server task should complete");
    }
}
