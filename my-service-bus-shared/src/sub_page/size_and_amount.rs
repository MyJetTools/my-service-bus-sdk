#[derive(Debug, Clone)]
pub struct SizeAndAmount {
    pub size: usize,
    pub amount: usize,
}

impl SizeAndAmount {
    pub fn new() -> Self {
        Self { size: 0, amount: 0 }
    }

    pub fn added(&mut self, size: usize) {
        self.size += size;
        self.amount += 1;
    }

    pub fn removed(&mut self, size: usize) {
        self.size -= size;
        self.amount -= 1;
    }

    pub fn added_page(&mut self, other: &SizeAndAmount) {
        self.size += other.size;
        self.amount += other.amount;
    }

    pub fn removed_page(&mut self, other: &SizeAndAmount) {
        self.size -= other.size;
        self.amount -= other.amount;
    }
}
