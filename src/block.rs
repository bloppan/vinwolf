use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TicketEnvelope {
    pub signature: String,
    pub attempt: u8,
}
// E ≡ (ET ,EV ,EP ,EA,EG)
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Extrinsic {
    pub tickets: Vec<TicketEnvelope>, // Tickets
//    ev: String, // Votes
//    ep: String, // Preimages
//    ea: String, // Availability
//    eg: String, // Reports
//    e: Vec<u8>, // Extrinsic vector serialized
}
/*
// H ≡ (Hp,Hr,Hx,Ht,He,Hw,Hj,Hk,Hv,Hs)
struct Header {
    hp: String, // Parent hash
    hr: String, // Prior state root
    hx: String, // Extrinsic hash
    ht: String, // Time slot index
    he: String, // Epoch
    hw: String, // Winning tickets
    hj: String, // Judgements
    hk: String, // Block author key
    hv: String, // VRF signature
    hs: String, // Block seal
    h: Vec<u8>, // Header vector serialized
}

// B ≡ (H,E)
struct Block {
    header: Header,
    extrinsic: Extrinsic,
}

impl Block {
    fn new(header: Header, extrinsic: Extrinsic) -> Block {
        Block { header, extrinsic }
    }
}

pub fn test_new_block() {

    let header = Header {
        hp: String::from("parent_hash_example"),
        hr: String::from("prior_state_root_example"),
        hx: String::from("extrinsic_hash_example"),
        ht: String::from("time_slot_index_example"),
        he: String::from("epoch_example"),
        hw: String::from("winning_tickets_example"),
        hj: String::from("judgements_example"),
        hk: String::from("block_author_key_example"),
        hv: String::from("vrf_signature_example"),
        hs: String::from("block_seal_example"),
        h: vec![0, 1, 2, 3, 4, 5],
    };

    // Crear una instancia de Extrinsic
    let extrinsic = Extrinsic {
        et: String::from("tickets_example"),
        ev: String::from("votes_example"),
        ep: String::from("preimages_example"),
        ea: String::from("availability_example"),
        eg: String::from("reports_example"),
        e: vec![6, 7, 8, 9, 10],
    };

    // Crear una instancia de Block utilizando la función `new`
    let block = Block::new(header, extrinsic);

    // Ejemplo de impresión para verificar el resultado
    println!("Block Header Parent Hash: {}", block.header.hp);
    println!("Block Header Prior State Root: {}", block.header.hr);
    println!("Block Header Extrinsic Hash: {}", block.header.hx);
    println!("Block Header Time Slot Index: {}", block.header.ht);
    println!("Block Header Epoch: {}", block.header.he);
    println!("Block Header Winning Tickets: {}", block.header.hw);
    println!("Block Header Judgements: {}", block.header.hj);
    println!("Block Header Block Author Key: {}", block.header.hk);
    println!("Block Header VRF Signature: {}", block.header.hv);
    println!("Block Header Block Seal: {}", block.header.hs);
    println!("Block Header Serialized: {:?}", block.header.h);

    println!("Block Extrinsic Tickets: {}", block.extrinsic.et);
    println!("Block Extrinsic Votes: {}", block.extrinsic.ev);
    println!("Block Extrinsic Preimages: {}", block.extrinsic.ep);
    println!("Block Extrinsic Availability: {}", block.extrinsic.ea);
    println!("Block Extrinsic Reports: {}", block.extrinsic.eg);
    println!("Block Extrinsic Serialized: {:?}", block.extrinsic.e);
}


*/