use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelProtocol {
    WebSocket,
    TCP,
    Stdio,
}

#[derive(Debug, Clone)]
pub struct TunnelConnection {
    pub id: String,
    pub remote_url: String,
    pub local_port: u16,
    pub protocol: TunnelProtocol,
}

#[derive(Debug, Clone)]
pub struct ReverseTunnel {
    tunnels: HashMap<String, TunnelConnection>,
}

impl ReverseTunnel {
    pub fn new() -> Self {
        ReverseTunnel {
            tunnels: HashMap::new(),
        }
    }

    pub fn add(&mut self, conn: TunnelConnection) -> Result<(), String> {
        let id = conn.id.clone();
        if self.tunnels.contains_key(&id) {
            return Err(format!("Tunnel '{}' already exists", id));
        }
        self.tunnels.insert(id, conn);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.tunnels.remove(id).is_some()
    }

    pub fn get(&self, id: &str) -> Option<&TunnelConnection> {
        self.tunnels.get(id)
    }

    pub fn list(&self) -> Vec<&TunnelConnection> {
        self.tunnels.values().collect()
    }

    pub fn len(&self) -> usize {
        self.tunnels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tunnels.is_empty()
    }
}

impl Default for ReverseTunnel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_tunnel() {
        let mut rt = ReverseTunnel::new();
        let conn = TunnelConnection {
            id: "t1".into(),
            remote_url: "ws://example.com/tunnel".into(),
            local_port: 8080,
            protocol: TunnelProtocol::WebSocket,
        };
        rt.add(conn.clone()).expect("value should be ok in test");
        assert!(rt.get("t1").is_some());
        assert_eq!(
            rt.get("t1").expect("value should be ok in test").local_port,
            8080
        );
    }

    #[test]
    fn test_remove_tunnel() {
        let mut rt = ReverseTunnel::new();
        rt.add(TunnelConnection {
            id: "t2".into(),
            remote_url: "tcp://remote:9000".into(),
            local_port: 9001,
            protocol: TunnelProtocol::TCP,
        })
        .expect("value should be ok in test");
        assert!(rt.remove("t2"));
        assert!(!rt.remove("t2"));
    }

    #[test]
    fn test_list_empty() {
        let rt = ReverseTunnel::new();
        assert!(rt.is_empty());
        assert_eq!(rt.len(), 0);
    }
}
