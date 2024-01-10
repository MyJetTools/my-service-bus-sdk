use std::collections::HashMap;

#[derive(Clone)]
pub struct PacketVersions {
    versions: [u8; 256],
}

impl PacketVersions {
    pub fn new() -> PacketVersions {
        PacketVersions {
            versions: [0u8; 256],
        }
    }

    pub fn get_packet_version(&self, packet_no: u8) -> u8 {
        self.versions[packet_no as usize]
    }

    pub fn update(&mut self, data: &HashMap<u8, i32>) {
        for (i, v) in data {
            self.set_packet_version(*i, *v as u8)
        }
    }

    pub fn set_packet_version(&mut self, packet: u8, value: u8) {
        self.versions[packet as usize] = value;
    }
}
