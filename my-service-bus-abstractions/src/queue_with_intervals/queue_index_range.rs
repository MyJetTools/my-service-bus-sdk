
pub enum QueueIndexRangeCompare {
    Below,
    Inside,
    Above,
}

pub enum RemoveResult{
    NoUpdate,
    InsertNew(QueueIndexRange),
    RemoveItem
}

#[derive(Debug, Clone)]
pub struct QueueIndexRange {
    pub from_id: i64,
    pub to_id: i64,
}

impl QueueIndexRange {
    pub fn restore(from_id: i64, to_id: i64) -> QueueIndexRange {
        QueueIndexRange { from_id, to_id }
    }

    pub fn new_empty(start_id: i64) -> QueueIndexRange {
        QueueIndexRange {
            from_id: start_id,
            to_id: start_id - 1,
        }
    }

    pub fn new_with_single_value(value: i64) -> QueueIndexRange {
        QueueIndexRange {
            from_id: value,
            to_id: value,
        }
    }

    pub fn try_join_with_the_next_one(&mut self, next_one: QueueIndexRange) -> bool {
        if self.to_id + 1 == next_one.from_id {
            self.to_id = next_one.to_id;
            return true;
        }

        return false;
    }

    pub fn is_in_my_interval(&self, id: i64)->bool{
        id >= self.from_id && id <= self.to_id
    }



    pub fn can_be_joined_to_interval_from_the_left(&self, id: i64)->bool{
       self.from_id -1 <= id && id <= self.to_id
    }

    pub fn can_be_joined_to_interval_from_the_right(&self, id: i64)->bool{
        self.from_id <= id && id <= self.to_id +1
    }

    pub fn is_my_interval_to_remove(&self, id: i64) -> bool {
        if self.is_empty() {
            panic!("We are trying to find interval to remove but we bumped empty interval");
        }

        id >= self.from_id && id <= self.to_id
    }

    pub fn init(&mut self) {
        self.to_id = self.from_id - 1;
    }

    pub fn remove(&mut self, message_id: i64) -> RemoveResult {


        if self.from_id == message_id && self.to_id == message_id{
            self.from_id += 1;

            return RemoveResult::RemoveItem;
        }


        if self.from_id == message_id {
            self.from_id += 1;
            return RemoveResult::NoUpdate;
        }

        if self.to_id == message_id {
            self.to_id -= 1;
            return RemoveResult::NoUpdate;
        }

        let new_item = QueueIndexRange {
            from_id: message_id + 1,
            to_id: self.to_id,
        };

        self.to_id = message_id - 1;

        return RemoveResult::InsertNew(new_item);
    }


    pub fn dequeue(&mut self) -> Option<i64> {
        if self.from_id > self.to_id {
            return None;
        }

        let result = self.from_id;
        self.from_id = self.from_id + 1;
        Some(result)
    }

    pub fn peek(&self) -> Option<i64> {
        if self.from_id > self.to_id {
            return None;
        }

        return Some(self.from_id);
    }

    pub fn enqueue(&mut self, id: i64) {
        if self.is_empty() {
            self.from_id = id;
            self.to_id = id;
            return;
        }

        if self.from_id >= id && self.to_id <= id {
            panic!(
                "Warning.... Something went wrong. We are enqueueing the Value {} which is already in the queue. Range: {:?}. ",
                id, self, 
            );
        } else if self.to_id + 1 == id {
            self.to_id = id;
        } else if self.from_id - 1 == id {
            self.from_id = id
        } else {
            panic!(
                "Something went wrong. Invalid interval is chosen to enqueue. Range: {:?}. NewValue: {}",
                self, id
            );
        }
    }

    pub fn try_merge_next(&mut self, next_item: &QueueIndexRange) -> bool {
        if self.to_id + 1 == next_item.from_id {
            self.to_id = next_item.to_id;
            return true;
        }

        return false;
    }

    pub fn try_join(&mut self, id_to_join: i64) -> bool {
        if self.is_empty() {
            self.from_id = id_to_join;
            self.to_id = id_to_join;
        }

        if id_to_join == self.from_id - 1 {
            self.from_id = id_to_join;
            return true;
        }

        if id_to_join == self.to_id + 1 {
            self.to_id = id_to_join;
            return true;
        }

        return false;
    }

    pub fn is_empty(&self) -> bool {
        self.to_id < self.from_id
    }

    pub fn is_before(&self, id: i64) -> bool {
        id < self.from_id - 1
    }

    pub fn compare_with(&self, id: i64) -> Option<QueueIndexRangeCompare> {
        if self.is_empty() {
            return None;
        }

        if id < self.from_id {
            return Some(QueueIndexRangeCompare::Below);
        }

        if id > self.to_id {
            return Some(QueueIndexRangeCompare::Above);
        }

        return Some(QueueIndexRangeCompare::Inside);
    }


    #[cfg(test)]
    pub fn to_string(&self) -> String {
        if self.is_empty() {
            return "EMPTY".to_string();
        }

        return format!("{} - {}", self.from_id, self.to_id);
    }

    pub fn len(&self) -> i64 {
        self.to_id - self.from_id + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let index_range = QueueIndexRange::new_empty(0);

        assert_eq!(index_range.len(), 0);
        assert_eq!(0, index_range.from_id);
        assert_eq!(-1, index_range.to_id);

        println!("{}", index_range.to_string());
    }

    #[test]
    fn test_one_enqueue_one_dequeue() {
        let mut index_range = QueueIndexRange::new_empty(0);

        index_range.enqueue(0);

        assert_eq!(index_range.len(), 1);
        assert_eq!(0, index_range.from_id);
        assert_eq!(0, index_range.to_id);

        let res = index_range.dequeue();

        assert_eq!(index_range.len(), 0);
        assert_eq!(1, index_range.from_id);
        assert_eq!(0, index_range.to_id);
        assert_eq!(0, res.unwrap());
    }

    #[test]
    fn test_two_enqueue_two_dequeue() {
        let mut index_range = QueueIndexRange::new_with_single_value(5);

        index_range.enqueue(6);

        assert_eq!(index_range.len(), 2);

        let res = index_range.dequeue();
        assert_eq!(5, res.unwrap());
        let res = index_range.dequeue();
        assert_eq!(6, res.unwrap());

        let res = index_range.dequeue();
        assert_eq!(true, res.is_none());
    }

    #[test]
    fn test_match_case() {
        let index_range = QueueIndexRange::restore(5, 10);

        let _result = index_range.compare_with(4).unwrap();
        assert_eq!(true, matches!(QueueIndexRangeCompare::Below, _result));

        let _result = index_range.compare_with(5).unwrap();
        assert_eq!(true, matches!(QueueIndexRangeCompare::Inside, _result));

        let _result = index_range.compare_with(10).unwrap();
        assert_eq!(true, matches!(QueueIndexRangeCompare::Inside, _result));

        let _result = index_range.compare_with(11).unwrap();
        assert_eq!(true, matches!(QueueIndexRangeCompare::Above, _result));
    }
}
