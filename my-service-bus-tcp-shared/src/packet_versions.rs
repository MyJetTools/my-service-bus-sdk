use std::collections::HashMap;

#[derive(Clone)]
pub struct PacketVersions {
    versions: Vec<i32>,
}

impl PacketVersions {
    pub fn new() -> PacketVersions {
        PacketVersions {
            versions: vec![0i32; 256],
        }
    }

    pub fn get_packet_version(&self, packet_no: u8) -> i32 {
        //Unsafe: we fill 256 values at new(). According to science - u8 can not be more than 255
        unsafe {
            return self.versions.get_unchecked(packet_no as usize).clone();
        }
    }

    pub fn update(&mut self, data: &HashMap<u8, i32>) {
        for (i, v) in data {
            self.versions[*i as usize] = *v
        }
    }

    pub fn set_packet_version(&mut self, packet: u8, value: i32) {
        self.versions[packet as usize] = value;
    }
}
