use std::collections::{HashMap, VecDeque};

use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Clone, Debug)]
pub struct LockItem {
    pub id: i64,
    pub data: VecDeque<String>,
    pub date: i64,
}

impl LockItem {
    pub fn to_string(&self) -> String {
        let mut result: Vec<u8> = Vec::new();

        for itm in &self.data {
            if result.len() > 0 {
                result.extend_from_slice("->".as_bytes());
            }

            result.extend_from_slice(itm.as_bytes());
        }

        String::from_utf8(result).unwrap()
    }

    fn do_exit(&mut self) -> bool {
        let index = self.data.len();
        self.data.remove(index - 1);
        return self.data.len() == 0;
    }
}

pub struct Locks {
    data: HashMap<i64, LockItem>,
}

impl Locks {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn new_lock(&mut self, id: i64, process: String) {
        if !self.data.contains_key(&id) {
            self.data.insert(
                id,
                LockItem {
                    id,
                    data: VecDeque::new(),
                    date: DateTimeAsMicroseconds::now().unix_microseconds,
                },
            );
        }

        self.data.get_mut(&id).unwrap().data.push_back(process);
    }

    pub fn exit(&mut self, id: i64) {
        let mut remove = false;

        {
            let found = self.data.get_mut(&id);

            if let Some(found) = found {
                remove = found.do_exit();
            }
        }

        if remove {
            self.data.remove(&id);
        }
    }

    pub fn get_all(&self) -> Vec<LockItem> {
        let mut result = Vec::new();

        for item in self.data.values() {
            result.push(item.clone());
        }

        result
    }
}
