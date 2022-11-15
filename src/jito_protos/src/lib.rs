use bincode::serialize;
use solana_sdk::transaction::VersionedTransaction;

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod block {
    tonic::include_proto!("block");
}

pub mod block_engine {
    tonic::include_proto!("block_engine");
}

pub mod bundle {
    tonic::include_proto!("bundle");
}

pub mod packet {
    tonic::include_proto!("packet");
}

pub mod relayer {
    tonic::include_proto!("relayer");
}

pub mod searcher {
    tonic::include_proto!("searcher");
}

pub mod shared {
    tonic::include_proto!("shared");
}

/// Converts a VersionedTransaction to a protobuf packet
pub fn proto_packet_from_versioned_tx(tx: &VersionedTransaction) -> packet::Packet {
    let data = serialize(tx).expect("serializes");
    let size = data.len() as u64;
    packet::Packet {
        data,
        meta: Some(packet::Meta {
            size,
            addr: "".to_string(),
            port: 0,
            flags: None,
            sender_stake: 0,
        }),
    }
}
