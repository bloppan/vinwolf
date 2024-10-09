use crate::types::{
    ValidatorIndex, Ed25519Key, Ed25519Signature, TimeSlot, 
    OpaqueHash, ServiceId, BandersnatchRingSignature, TicketAttempt
};
use crate::globals::{VALIDATORS_SUPER_MAJORITY, AVAIL_BITFIELD_BYTES};
use crate::codec::{Encode, EncodeSize, EncodeLen, Decode, DecodeLen, ReadError, BytesReader};
use crate::work::package::WorkReport;

// The extrinsic data is split into its several portions:
//     Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring.
//     Votes, by validators, on dispute(s) arising between them presently taking place.
//     Static data which is presently being requested to be available for workloads to be able to fetch on demand.
//     Assurances by each validator concerning which of the input data of workloads they have correctly received and are 
//     storing locally.
//     Reports of newly completed workloads whose accuracy is guaranteed by specific validators.

pub struct Extrinsic {
    tickets: TicketsExtrinsic,
    disputes: DisputesExtrinsic,
    preimages: PreimagesExtrinsic,
    assurances: AssurancesExtrinsic,
    guarantees: GuaranteesExtrinsic,
}

impl Extrinsic {
    pub fn decode(extrinsic_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let tickets = TicketsExtrinsic::decode(extrinsic_blob)?;
        let disputes = DisputesExtrinsic::decode(extrinsic_blob)?;
        let preimages = PreimagesExtrinsic::decode(extrinsic_blob)?;
        let assurances = AssurancesExtrinsic::decode(extrinsic_blob)?;
        let guarantees = GuaranteesExtrinsic::decode(extrinsic_blob)?;

        Ok(Extrinsic {
            tickets,
            disputes,
            preimages,
            assurances,
            guarantees,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut extrinsic_blob: Vec<u8> = Vec::new();

        self.tickets.encode_to(&mut extrinsic_blob);
        self.disputes.encode_to(&mut extrinsic_blob);
        self.preimages.encode_to(&mut extrinsic_blob);
        self.assurances.encode_to(&mut extrinsic_blob);
        self.guarantees.encode_to(&mut extrinsic_blob);

        return extrinsic_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

// The guarantees extrinsic is a series of guarantees, at most one for each core, each of which is 
// a tuple of a work-report, a credential and its corresponding timeslot. The core index of each 
// guarantee must be unique and guarantees must be in ascending order of this.
// They are reports of newly completed workloads whose accuracy is guaranteed by specific validators. 
// A work-package, which comprises several work items, is transformed by validators acting as guarantors 
// into its corresponding workreport, which similarly comprises several work outputs and then presented 
// on-chain within the guarantees extrinsic.

pub struct GuaranteesExtrinsic {
    report_guarantee: Vec<ReportGuarantee>,
}

struct ReportGuarantee {
    report: WorkReport,
    slot: TimeSlot,
    signatures: Vec<ValidatorSignature>,
}

struct ValidatorSignature {
    validator_index: ValidatorIndex,
    signature: Ed25519Signature,
}

impl GuaranteesExtrinsic {

    pub fn decode(guarantees_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let num_guarantees = guarantees_blob.read_byte()? as usize;
        let mut report_guarantee: Vec<ReportGuarantee> = Vec::with_capacity(num_guarantees);
        for _ in 0..num_guarantees {
            let report = WorkReport::decode(guarantees_blob)?;
            let slot = TimeSlot::decode(guarantees_blob)?;
            let num_signatures = guarantees_blob.read_byte()? as usize;
            let mut signatures: Vec<ValidatorSignature> = Vec::with_capacity(num_signatures);
            for _ in 0..num_signatures {
                let validator_index = ValidatorIndex::decode(guarantees_blob)?;
                let signature = Ed25519Signature::decode(guarantees_blob)?;
                signatures.push(ValidatorSignature{validator_index, signature});
            }
            report_guarantee.push(ReportGuarantee{report, slot, signatures});
        }
        Ok(GuaranteesExtrinsic{ report_guarantee })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut guarantees_blob: Vec<u8> = Vec::new();
        guarantees_blob.push(self.report_guarantee.len() as u8);
        for guarantee in &self.report_guarantee {
            let _ = guarantee.report.encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            guarantees_blob.push(guarantee.signatures.len() as u8);
            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }
        return guarantees_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

// The assurances extrinsic are the input data of workloads they have correctly received and are storing locally.
// The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a 
// sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the 
// validator who is assuring. A value of 1 at any given index implies that the validator assures they are contributing 
// to its availability.

pub struct AssurancesExtrinsic {
    assurances: Vec<AvailAssurance>, 
}

struct AvailAssurance {
    anchor: OpaqueHash,
    bitfield: [u8; AVAIL_BITFIELD_BYTES],
    validator_index: ValidatorIndex,
    signature: Ed25519Signature,
}

impl AssurancesExtrinsic {

    pub fn decode(assurances_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_assurances = assurances_blob.read_byte()? as usize;
        let mut assurances = Vec::with_capacity(num_assurances);  
        for _ in 0..num_assurances {
            let anchor = OpaqueHash::decode(assurances_blob)?;
            let bitfield = <[u8; AVAIL_BITFIELD_BYTES]>::decode(assurances_blob)?;
            let validator_index = ValidatorIndex::decode(assurances_blob)?;
            let signature = Ed25519Signature::decode(assurances_blob)?;
            assurances.push(AvailAssurance {
                anchor,
                bitfield,
                validator_index,
                signature,
            });
        }
        Ok(AssurancesExtrinsic { assurances })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut assurances_blob: Vec<u8> = Vec::new();
        assurances_blob.push(self.assurances.len() as u8);
        for assurance in &self.assurances {
            assurance.anchor.encode_to(&mut assurances_blob);
            assurance.bitfield.encode_to(&mut assurances_blob);
            assurance.validator_index.encode_size(2).encode_to(&mut assurances_blob);
            assurance.signature.encode_to(&mut assurances_blob);
        }
        return assurances_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

// Preimages are static data which is presently being requested to be available for workloads to be able to 
// fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 
// The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and without 
// duplicates. The data must have been solicited by a service but not yet be provided.

pub struct PreimagesExtrinsic {
    preimages: Vec<Preimage>,
}

struct Preimage {
    requester: ServiceId,
    blob: Vec<u8>,
}

impl PreimagesExtrinsic {

    pub fn decode(preimage_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_preimages = preimage_blob.read_byte()? as usize;
        let mut preimg_extrinsic: Vec<Preimage> = Vec::with_capacity(num_preimages);
        for _ in 0..num_preimages {
            let requester = ServiceId::decode(preimage_blob)?;
            let blob = Vec::<u8>::decode_len(preimage_blob)?;
            preimg_extrinsic.push(Preimage { requester, blob });
        }
        Ok(PreimagesExtrinsic { preimages: preimg_extrinsic })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut preimg_encoded = Vec::with_capacity(std::mem::size_of::<PreimagesExtrinsic>());
        preimg_encoded.push(self.preimages.len() as u8);
        for preimage in &self.preimages {
            preimage.requester.encode_to(&mut preimg_encoded);
            preimage.blob.as_slice().encode_len().encode_to(&mut preimg_encoded);
        }
        return preimg_encoded;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors’ assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached.

struct Judgement {
    vote: bool,
    index: ValidatorIndex,
    signature: Ed25519Signature,
}

impl Judgement {
    fn decode_len(judgement_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
        for _ in 0..VALIDATORS_SUPER_MAJORITY {
            let vote: bool = judgement_blob.read_byte()? != 0;
            let index = ValidatorIndex::decode(judgement_blob)?;
            let signature = Ed25519Signature::decode(judgement_blob)?; 
            let judgement = Judgement {vote, index, signature};
            votes.push(judgement);
        }
        Ok(votes)
    }

    fn encode_len(judgments: &[Judgement]) -> Vec<u8> {
        let mut judgement_blob: Vec<u8> = Vec::new();
        for j in 0..VALIDATORS_SUPER_MAJORITY {
            judgement_blob.push(judgments[j as usize].vote as u8);
            judgments[j as usize].index.encode_size(2).encode_to(&mut judgement_blob);
            judgments[j as usize].signature.encode_to(&mut judgement_blob);
        }
        return judgement_blob;
    }
}

// A Verdict is a compilation of judgments coming from exactly two-thirds plus one of either the active validator set 
// or the previous epoch’s validator set, i.e. the Ed25519 keys of κ or λ. Verdicts contains only the report hash and 
// the sum of positive judgments. We require this total to be either exactly two-thirds-plus-one, zero or one-third 
// of the validator set indicating, respectively, that the report is good, that it’s bad, or that it’s wonky.

struct Verdict {
    target: OpaqueHash,
    age: u32,
    votes: Vec<Judgement>,
}

impl Verdict {
    fn decode_len(verdict_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_verdicts = verdict_blob.read_byte()? as usize;
        let mut verdicts: Vec<Verdict> = Vec::with_capacity(num_verdicts);
        for _ in 0..num_verdicts {
            let target = OpaqueHash::decode(verdict_blob)?;
            let age = u32::decode(verdict_blob)?;
            let votes = Judgement::decode_len(verdict_blob)?;
            verdicts.push(Verdict {target, age, votes});
        }
        Ok(verdicts)
    }

    fn encode_len(verdicts: &[Verdict]) -> Vec<u8> {
        let mut verdicts_blob: Vec<u8> = Vec::new();
        verdicts_blob.push(verdicts.len() as u8);
        for verdict in verdicts {
            verdict.target.encode_to(&mut verdicts_blob);
            verdict.age.encode_size(4).encode_to(&mut verdicts_blob);
            Judgement::encode_len(&verdict.votes).encode_to(&mut verdicts_blob);
        }
        return verdicts_blob;
    }
}

// A culprit is a proofs of the misbehavior of one or more validators by guaranteeing a work-report found to be invalid.
// Is a offender signature.

struct Culprit {
    target: OpaqueHash,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Culprit {
    fn decode_len(culprit_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_culprits = culprit_blob.read_byte()? as usize; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);
        for _ in 0..num_culprits {
            let target = OpaqueHash::decode(culprit_blob)?;
            let key = Ed25519Key::decode(culprit_blob)?; 
            let signature = Ed25519Signature::decode(culprit_blob)?;
            let culprit = Culprit {target, key, signature};
            culprits.push(culprit);
        }
        Ok(culprits)
    }

    fn encode(&self) -> Vec<u8> {
        let mut culprit_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Culprit>());
        self.encode_to(&mut culprit_blob);
        return culprit_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.target.encode()); 
        into.extend_from_slice(&self.key.encode()); 
        into.extend_from_slice(&self.signature.encode()); 
    }

    fn encode_len(culprits: &[Culprit]) -> Vec<u8> {
        let mut culprits_len: Vec<u8> = Vec::with_capacity(culprits.len() * std::mem::size_of::<Fault>());
        culprits_len.push(culprits.len() as u8); 
        for culprit in culprits {
            culprits_len.extend_from_slice(&culprit.encode());
        }
        return culprits_len;
    }
}

// A fault is a proofs of the misbehavior of one or more validators by signing a judgment found to be contradiction to a 
// work-report’s validity. Is a offender signature. Must be ordered by validators Ed25519Key. There may be no duplicate
// report hashes within the extrinsic, nor amongst any past reported hashes.

struct Fault {
    target: OpaqueHash,
    vote: bool,
    key: Ed25519Key,
    signature: Ed25519Signature,
}

impl Fault {
    fn decode_len(faults_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        let num_faults = faults_blob.read_byte()? as usize;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);
        for _ in 0..num_faults {
            let target = OpaqueHash::decode(faults_blob)?; 
            let vote: bool = faults_blob.read_byte()? != 0;
            let key = OpaqueHash::decode(faults_blob)?;
            let signature = Ed25519Signature::decode(faults_blob)?;
            let fault = Fault {target, vote, key, signature};
            faults.push(fault);
        }
        Ok(faults)
    }

    fn encode(&self) -> Vec<u8> {
        let mut fault_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Fault>());
        self.encode_to(&mut fault_blob);
        return fault_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.target.encode()); 
        into.extend_from_slice(&self.vote.encode()); 
        into.extend_from_slice(&self.key.encode()); 
        into.extend_from_slice(&self.signature.encode()); 
    }

    fn encode_len(faults: &[Fault]) -> Vec<u8> {
        let mut faults_len: Vec<u8> = Vec::with_capacity(faults.len() * std::mem::size_of::<Fault>());
        faults_len.push(faults.len() as u8); 
        for fault in faults {
            faults_len.extend_from_slice(&fault.encode());
        }
        return faults_len;
    }
}

// The disputes extrinsic may contain one or more verdicts v as a compilation of judgments coming from exactly 
// two-thirds plus one of either the active validator set or the previous epoch’s validator set, i.e. the Ed25519 
// keys of κ or λ. Additionally, it may contain proofs of the misbehavior of one or more validators, either by 
// guaranteeing a work-report found to be invalid (culprits), or by signing a judgment found to be contradiction 
// to a work-report’s validity (faults). Both are considered a kind of offense.

pub struct DisputesExtrinsic {
    verdicts: Vec<Verdict>,
    culprits: Vec<Culprit>,
    faults: Vec<Fault>,
}

impl DisputesExtrinsic {
    pub fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let verdicts = Verdict::decode_len(dispute_blob)?;
        let culprits = Culprit::decode_len(dispute_blob)?;
        let faults = Fault::decode_len(dispute_blob)?;

        Ok(DisputesExtrinsic {
            verdicts,
            culprits,
            faults,
        })
    }

    pub fn encode(&self) -> Vec<u8> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        
        Verdict::encode_len(&self.verdicts).encode_to(&mut dispute_blob);
        Culprit::encode_len(&self.culprits).encode_to(&mut dispute_blob);
        Fault::encode_len(&self.faults).encode_to(&mut dispute_blob);
        
        return dispute_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

// Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” 
// to determine which validators are privileged to author a block for each timeslot in the following epoch. 
// Tickets specify an entry index together with a proof of ticket’s validity. The proof implies a ticket identifier, 
// a high-entropy unbiasable 32-octet sequence, which is used both as a score in the aforementioned contest and as 
// input to the on-chain vrf. 
// Towards the end of the epoch (i.e. Y slots from the start) this contest is closed implying successive blocks 
// within the same epoch must have an empty tickets extrinsic. At this point, the following epoch’s seal key sequence 
// becomes fixed. 
// We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index 
// (a natural number less than N) and a proof of ticket validity.

pub struct TicketsExtrinsic { 
    tickets: Vec<TicketEnvelope>,
}

pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    pub signature: BandersnatchRingSignature,
}

impl TicketsExtrinsic {
    pub fn decode(ticket_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_tickets = ticket_blob.read_byte()? as usize;
        let mut ticket_envelop = Vec::with_capacity(num_tickets);
        for _ in 0..num_tickets {
            let attempt = TicketAttempt::decode(ticket_blob)?;
            let signature = BandersnatchRingSignature::decode(ticket_blob)?;
            ticket_envelop.push(TicketEnvelope{ attempt, signature });
        }      
        Ok(TicketsExtrinsic { tickets: ticket_envelop })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut ticket_blob: Vec<u8> = Vec::new();
        self.encode_len().encode_to(&mut ticket_blob);
        return ticket_blob;
    }

    fn encode_len(&self) -> Vec<u8> {
        let num_tickets = self.tickets.len();
        let mut ticket_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<TicketEnvelope>() * num_tickets);
        ticket_blob.push(num_tickets as u8);
        for ticket in &self.tickets {
            ticket.attempt.encode_to(&mut ticket_blob);
            ticket.signature.encode_to(&mut ticket_blob);
        }
        return ticket_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}
