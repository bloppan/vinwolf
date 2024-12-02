use crate::types::{
    TimeSlot, OpaqueHash, ValidatorIndex, Ed25519Signature, Ed25519Key, 
    WorkReportHash, OffendersMark, Ed25519Public};
use crate::constants::{VALIDATORS_SUPER_MAJORITY, CORES_COUNT};
use crate::codec::{Encode, EncodeSize, Decode, DecodeLen, BytesReader, ReadError};
use crate::codec::{encode_unsigned, decode_unsigned};
use crate::codec::work_report::WorkReport;
use crate::codec::safrole::ValidatorData;

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesState {
    pub psi: DisputesRecords,
    pub rho: AvailabilityAssignments,
    pub tau: TimeSlot,
    pub kappa: Vec<ValidatorData>,
    pub lambda: Vec<ValidatorData>,
}

impl Encode for DisputesState {

    fn encode(&self) -> Vec<u8> {

        let mut state_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.psi.encode_to(&mut state_blob);
        self.rho.encode_to(&mut state_blob);
        self.tau.encode_to(&mut state_blob);
        ValidatorData::encode_all(&self.kappa).encode_to(&mut state_blob);
        ValidatorData::encode_all(&self.lambda).encode_to(&mut state_blob);

        return state_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for DisputesState {

    fn decode(state_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesState{
            psi: DisputesRecords::decode(state_blob)?,
            rho: AvailabilityAssignments::decode(state_blob)?,
            tau: TimeSlot::decode(state_blob)?,
            kappa: ValidatorData::decode_all(state_blob)?,
            lambda: ValidatorData::decode_all(state_blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesRecords {
    pub good: Vec<WorkReportHash>,
    pub bad: Vec<WorkReportHash>,
    pub wonky: Vec<WorkReportHash>,
    pub offenders: Vec<Ed25519Public>,
}

impl Encode for DisputesRecords {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        encode_unsigned(self.good.len()).encode_to(&mut blob);
        self.good.encode_to(&mut blob);
        encode_unsigned(self.bad.len()).encode_to(&mut blob);
        self.bad.encode_to(&mut blob);
        encode_unsigned(self.wonky.len()).encode_to(&mut blob);
        self.wonky.encode_to(&mut blob);
        encode_unsigned(self.offenders.len()).encode_to(&mut blob);
        self.offenders.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for DisputesRecords {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesRecords {
            good: Vec::<WorkReportHash>::decode_len(blob)?,
            bad: Vec::<WorkReportHash>::decode_len(blob)?,
            wonky: Vec::<WorkReportHash>::decode_len(blob)?,
            offenders: Vec::<Ed25519Public>::decode_len(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityAssignment {
    pub report: WorkReport,
    pub timeout: u32,
}

impl Encode for AvailabilityAssignment {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.report.encode_to(&mut blob);
        self.timeout.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignment {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(AvailabilityAssignment {
            report: WorkReport::decode(blob)?,
            timeout: u32::decode(blob)?,
        })
    }
}

pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;

impl Encode for AvailabilityAssignmentsItem {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<AvailabilityAssignment>());

        match self {
            None => {
                blob.push(0);
            }
            Some(assignment) => {
                blob.push(1);
                assignment.encode_to(&mut blob);
            }
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignmentsItem {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let option = blob.read_byte()?;
        match option {
            0 => Ok(None),
            1 => {
                let assignment = AvailabilityAssignment::decode(blob)?;
                Ok(Some(assignment))
            }
            _ => Err(ReadError::InvalidData),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailabilityAssignments {
    pub assignments: Vec<AvailabilityAssignmentsItem>,
}

impl Encode for AvailabilityAssignments {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<AvailabilityAssignmentsItem>() * CORES_COUNT);

        for assigment in &self.assignments {
            assigment.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignments {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut assignments = AvailabilityAssignments{assignments: Vec::with_capacity(std::mem::size_of::<AvailabilityAssignmentsItem>() * CORES_COUNT)};
        
        for _ in 0..CORES_COUNT {
            assignments
                .assignments
                .push(AvailabilityAssignmentsItem::decode(blob)?);
        }

        Ok(assignments)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputData {
    pub offenders_mark: OffendersMark,
}

impl Encode for OutputData {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<OffendersMark>() * self.offenders_mark.len());

        encode_unsigned(self.offenders_mark.len()).encode_to(&mut blob);

        for mark in &self.offenders_mark {
            mark.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputData {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputData {
            offenders_mark: {
                let num_offenders = decode_unsigned(blob)?;
                let mut offenders_mark: OffendersMark = Vec::with_capacity(num_offenders);
                for _ in 0..num_offenders {
                    offenders_mark.push(Ed25519Public::decode(blob)?);
                }
                offenders_mark
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    AlreadyJudged = 0,
    BadVoteSplit = 1,
    VerdictsNotSortedUnique = 2,
    JudgementsNotSortedUnique = 3,
    CulpritsNotSortedUnique = 4,
    FaultsNotSortedUnique = 5,
    NotEnoughCulprits = 6,
    NotEnoughFaults = 7,
    CulpritsVerdictNotBad = 8,
    FaultVerdictWrong = 9,
    OffenderAlreadyReported = 10,
    BadJudgementAge = 11,
    BadValidatorIndex = 12,
    BadSignature = 13,
    DisputeStateNotInitialized = 14,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputDisputes {
    Ok(OutputData),
    Err(ErrorCode),
}

impl Encode for OutputDisputes {

    fn encode(&self) -> Vec<u8> {

        let mut output_blob: Vec<u8> = Vec::new();

        match self {
            OutputDisputes::Ok(output_data) => {
                output_blob.push(0); // 0 = OK
                output_data.encode_to(&mut output_blob);
            }
            OutputDisputes::Err(error_code) => {
                output_blob.push(1); // 1 = ERROR
                output_blob.push(*error_code as u8); 
            }
        }

        return output_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDisputes {

    fn decode(output_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let result = output_blob.read_byte()?;
        if result == 0 {
            Ok(OutputDisputes::Ok(OutputData::decode(output_blob)?))  
        } else if result == 1 {
            let error_type = output_blob.read_byte()?;
            let error = match error_type {
                0 => ErrorCode::AlreadyJudged,
                1 => ErrorCode::BadVoteSplit,
                2 => ErrorCode::VerdictsNotSortedUnique,
                3 => ErrorCode::JudgementsNotSortedUnique,
                4 => ErrorCode::CulpritsNotSortedUnique,
                5 => ErrorCode::FaultsNotSortedUnique,
                6 => ErrorCode::NotEnoughCulprits,
                7 => ErrorCode::NotEnoughFaults,
                8 => ErrorCode::CulpritsVerdictNotBad,
                9 => ErrorCode::FaultVerdictWrong,
                10 => ErrorCode::OffenderAlreadyReported,
                11 => ErrorCode::BadJudgementAge,
                12 => ErrorCode::BadValidatorIndex,
                13 => ErrorCode::BadSignature,
                14 => ErrorCode::DisputeStateNotInitialized,
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputDisputes::Err(error))
        } else {
            return Err(ReadError::InvalidData);
        }
    }
}

// The disputes extrinsic may contain one or more verdicts v as a compilation of judgments coming from exactly 
// two-thirds plus one of either the active validator set or the previous epoch’s validator set, i.e. the Ed25519 
// keys of κ or λ. Additionally, it may contain proofs of the misbehavior of one or more validators, either by 
// guaranteeing a work-report found to be invalid (culprits), or by signing a judgment found to be contradiction 
// to a work-report’s validity (faults). Both are considered a kind of offense.

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesExtrinsic {
    pub verdicts: Vec<Verdict>,
    pub culprits: Vec<Culprit>,
    pub faults: Vec<Fault>,
}

impl Encode for DisputesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut dispute_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<DisputesExtrinsic>());
        
        Verdict::encode_len(&self.verdicts).encode_to(&mut dispute_blob);
        Culprit::encode_len(&self.culprits).encode_to(&mut dispute_blob);
        Fault::encode_len(&self.faults).encode_to(&mut dispute_blob);
        
        return dispute_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for DisputesExtrinsic {

    fn decode(dispute_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(DisputesExtrinsic {
            verdicts : Verdict::decode_len(dispute_blob)?,
            culprits : Culprit::decode_len(dispute_blob)?,
            faults : Fault::decode_len(dispute_blob)?,
        })
    }
}

impl DisputesExtrinsic {

    pub fn is_empty(&self) -> bool {
        self.verdicts.is_empty()
        && self.culprits.is_empty()
        && self.faults.is_empty()
    }
}
// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors’ assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached.

#[derive(Debug, Clone, PartialEq)]
pub struct Judgement {
    pub vote: bool,
    pub index: ValidatorIndex,
    pub signature: Ed25519Signature,
}

impl Judgement {

    fn encode_len(judgments: &[Judgement]) -> Vec<u8> {
        
        let mut judgement_blob: Vec<u8> = Vec::new();
        
        for judgment in judgments.iter().take(VALIDATORS_SUPER_MAJORITY) {
            judgement_blob.push(judgment.vote as u8);
            judgment.index.encode_size(2).encode_to(&mut judgement_blob);
            judgment.signature.encode_to(&mut judgement_blob);
        }
        
        
        return judgement_blob;
    }

    fn decode_len(judgement_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {
        
        let mut votes: Vec<Judgement> = Vec::with_capacity(VALIDATORS_SUPER_MAJORITY);
        
        for _ in 0..VALIDATORS_SUPER_MAJORITY {
            let vote: bool = judgement_blob.read_byte()? != 0;
            let index = ValidatorIndex::decode(judgement_blob)?;
            let signature = Ed25519Signature::decode(judgement_blob)?; 
            votes.push(Judgement{vote, index, signature});
        }
        
        Ok(votes)
    }
}

// A Verdict is a compilation of judgments coming from exactly two-thirds plus one of either the active validator set 
// or the previous epoch’s validator set, i.e. the Ed25519 keys of κ or λ. Verdicts contains only the report hash and 
// the sum of positive judgments. We require this total to be either exactly two-thirds-plus-one, zero or one-third 
// of the validator set indicating, respectively, that the report is good, that it’s bad, or that it’s wonky.

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub target: OpaqueHash,
    pub age: u32,
    pub votes: Vec<Judgement>,
}

impl Verdict {

    fn decode_len(verdict_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_verdicts = decode_unsigned(verdict_blob)?;
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
        encode_unsigned(verdicts.len()).encode_to(&mut verdicts_blob);

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

#[derive(Debug, Clone, PartialEq)]
pub struct Culprit {
    pub target: OpaqueHash,
    pub key: Ed25519Key,
    pub signature: Ed25519Signature,
}

impl Encode for Culprit {

    fn encode(&self) -> Vec<u8> {
        
        let mut culprit_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Culprit>());

        self.target.encode_to(&mut culprit_blob);
        self.key.encode_to(&mut culprit_blob);
        self.signature.encode_to(&mut culprit_blob);
        
        return culprit_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Culprit {

    fn decode(culprit_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Culprit {
            target: OpaqueHash::decode(culprit_blob)?,
            key: Ed25519Key::decode(culprit_blob)?,
            signature : Ed25519Signature::decode(culprit_blob)?,
        })
    }
}

impl Culprit {

    fn encode_len(culprits: &[Culprit]) -> Vec<u8> {
        
        let mut culprits_blob: Vec<u8> = Vec::with_capacity(culprits.len() * std::mem::size_of::<Fault>());
        encode_unsigned(culprits.len()).encode_to(&mut culprits_blob); 
        
        for culprit in culprits {
            culprit.encode_to(&mut culprits_blob);
        }
        
        return culprits_blob;
    }

    fn decode_len(culprits_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_culprits = decode_unsigned(culprits_blob)?; 
        let mut culprits: Vec<Culprit> = Vec::with_capacity(num_culprits);

        for _ in 0..num_culprits {
            culprits.push(Culprit::decode(culprits_blob)?);
        }

        Ok(culprits)
    }
}

// A fault is a proofs of the misbehavior of one or more validators by signing a judgment found to be contradiction to a 
// work-report’s validity. Is a offender signature. Must be ordered by validators Ed25519Key. There may be no duplicate
// report hashes within the extrinsic, nor amongst any past reported hashes.

#[derive(Debug, Clone, PartialEq)]
pub struct Fault {
    pub target: OpaqueHash,
    pub vote: bool,
    pub key: Ed25519Key,
    pub signature: Ed25519Signature,
}

impl Encode for Fault {

    fn encode(&self) -> Vec<u8> {

        let mut fault_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Fault>());
        
        self.target.encode_to(&mut fault_blob);
        self.vote.encode_to(&mut fault_blob);
        self.key.encode_to(&mut fault_blob);
        self.signature.encode_to(&mut fault_blob);

        return fault_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());       
    }
}

impl Decode for Fault {

    fn decode(fault_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Fault {
            target: OpaqueHash::decode(fault_blob)?,
            vote: fault_blob.read_byte()? != 0,
            key: OpaqueHash::decode(fault_blob)?,
            signature: Ed25519Signature::decode(fault_blob)?,
        })
    }
}

impl Fault {

    fn encode_len(faults: &[Fault]) -> Vec<u8> {

        let faults_len = faults.len();
        let mut faults_blob: Vec<u8> = Vec::with_capacity(faults_len * std::mem::size_of::<Fault>());
        encode_unsigned(faults_len).encode_to(&mut faults_blob);

        for fault in faults {
            fault.encode_to(&mut faults_blob);
        }

        return faults_blob;
    }

    fn decode_len(faults_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_faults = decode_unsigned(faults_blob)?;
        let mut faults: Vec<Fault> = Vec::with_capacity(num_faults);

        for _ in 0..num_faults {
            faults.push(Fault::decode(faults_blob)?);
        }

        Ok(faults)
    }
   
}

