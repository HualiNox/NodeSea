use rand::RngExt;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeID,
    pub address: String,
    pub port: u16,
}

impl Node {
    pub fn new(address: String, port: u16) -> Self {
        Self {
            id: NodeID::new(),
            address,
            port,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeID([u8; 20]);

impl NodeID {
    pub fn new() -> Self {
        let mut id = [0u8; 20];
        rand::rng().fill(&mut id);
        Self(id)
    }

    pub fn get_distance(&self, other: &NodeID) -> [u8; 20] {
        let mut distance = [0u8; 20];
        for ((a, b), c) in self.0.iter().zip(other.0.iter()).zip(distance.iter_mut()) {
            *c = a ^ b;
        }

        distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let id = NodeID::new();
        assert_eq!(id.0.len(), 20);
    }

    #[test]
    fn test_distance() {
        let id1 = NodeID([0; 20]);

        let mut data = [0u8; 20];
        data[19] = 1;

        let id2 = NodeID(data);

        assert_eq!(id1.get_distance(&id2), data);
    }
}
