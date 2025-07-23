use codec::{Encode, Decode, BytesReader};
use jam_types::{
    BandersnatchVrfSignature, HeaderHash, OpaqueHash, TimeSlot, TicketsMark, EpochMark, ValidatorIndex, Ed25519Public, Ed25519Signature, AvailAssurance, ReadError,
    TicketEnvelope, ReportGuarantee, Preimage, Judgement
};
pub mod header;
pub mod extrinsic;

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub unsigned: UnsignedHeader,
    pub seal: BandersnatchVrfSignature,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnsignedHeader {
    pub parent: HeaderHash,
    pub parent_state_root: OpaqueHash,
    pub extrinsic_hash: OpaqueHash,
    pub slot: TimeSlot,
    pub epoch_mark: Option<EpochMark>,
    pub tickets_mark: Option<TicketsMark>,
    pub offenders_mark: Vec<Ed25519Public>,
    pub author_index: ValidatorIndex,
    pub entropy_source: BandersnatchVrfSignature,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Extrinsic {
    // Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring.
    pub tickets: TicketsExtrinsic,
    // Votes, by validators, on dispute(s) arising between them presently taking place.
    pub disputes: DisputesExtrinsic,
    // Static data which is presently being requested to be available for workloads to be able to fetch on demand.
    pub preimages: PreimagesExtrinsic,
    // Assurances by each validator concerning which of the input data of workloads they have correctly received and are storing locally.
    pub assurances: AssurancesExtrinsic,
    // Reports of newly completed workloads whose accuracy is guaranteed by specific validators.
    pub guarantees: GuaranteesExtrinsic,
}
// The assurances extrinsic are the input data of workloads they have correctly received and are storing locally.
// The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a 
// sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the 
// validator who is assuring. A value of 1 at any given index implies that the validator assures they are contributing 
// to its availability.
#[derive(Debug, Clone, PartialEq)]
pub struct AssurancesExtrinsic {
    pub assurances: Vec<AvailAssurance>, 
}
#[derive(Debug, Clone, PartialEq)]
pub struct GuaranteesExtrinsic {
    pub report_guarantee: Vec<ReportGuarantee>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TicketsExtrinsic { 
    pub tickets: Vec<TicketEnvelope>,
}
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, std::hash::Hash)]
pub struct PreimagesExtrinsic {
    pub preimages: Vec<Preimage>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DisputesExtrinsic {
    pub verdicts: Vec<Verdict>,
    pub culprits: Vec<Culprit>,
    pub faults: Vec<Fault>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub target: OpaqueHash,
    pub age: u32,
    pub votes: Vec<Judgement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Culprit {
    pub target: OpaqueHash,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fault {
    pub target: OpaqueHash,
    pub vote: bool,
    pub key: Ed25519Public,
    pub signature: Ed25519Signature,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub header: Header,
    pub extrinsic: Extrinsic,
}

impl Default for Block {
    fn default() -> Self {
        Self { header: Header::default(), extrinsic: Extrinsic::default(), }
    }
}


impl Default for TicketsExtrinsic {
    fn default() -> Self {
        TicketsExtrinsic {
            tickets: Vec::new(),
        }
    }
}

impl Default for PreimagesExtrinsic {
    fn default() -> Self {
        PreimagesExtrinsic {
            preimages: Vec::new(),
        }
    }
}

impl Encode for Block {

    fn encode(&self) -> Vec<u8> {

        let mut block_blob: Vec<u8> = Vec::new();

        self.header.encode_to(&mut block_blob);
        self.extrinsic.encode_to(&mut block_blob);

        return block_blob;
    }
    
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    } 
}

impl Decode for Block {

    fn decode(block_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let header = Header::decode(block_blob)?;
        let extrinsic = Extrinsic::decode(block_blob)?;

        Ok(Block { header, extrinsic })
    }
}