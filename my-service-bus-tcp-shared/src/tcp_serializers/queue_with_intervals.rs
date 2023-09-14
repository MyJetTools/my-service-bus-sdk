use my_service_bus_abstractions::queue_with_intervals::QueueIndexRange;
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

pub fn serialize(payload: &mut Vec<u8>, value: &Vec<QueueIndexRange>) {
    super::i32::serialize(payload, value.len() as i32);

    for itm in value {
        super::i64::serialize(payload, itm.from_id);
        super::i64::serialize(payload, itm.to_id);
    }
}

pub async fn deserialize<T: SocketReader>(
    reader: &mut T,
) -> Result<Vec<QueueIndexRange>, ReadingTcpContractFail> {
    let len = reader.read_i32().await?;

    let mut result: Vec<QueueIndexRange> = Vec::new();

    for _ in 0..len {
        let from_id = reader.read_i64().await?;
        let to_id = reader.read_i64().await?;

        result.push(QueueIndexRange { from_id, to_id });
    }

    Ok(result)
}
