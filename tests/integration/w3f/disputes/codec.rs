use vinwolf::types::{DisputesRecords, AvailabilityAssignments, TimeSlot, ValidatorsData, DisputesErrorCode, OutputDataDisputes};
use vinwolf::utils::codec::{Encode, Decode, BytesReader, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct DisputesState {
    pub psi: DisputesRecords,
    pub rho: AvailabilityAssignments,
    pub tau: TimeSlot,
    pub kappa: ValidatorsData,
    pub lambda: ValidatorsData,
}

impl Encode for DisputesState {

    fn encode(&self) -> Vec<u8> {

        let mut state_blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.psi.encode_to(&mut state_blob);
        self.rho.encode_to(&mut state_blob);
        self.tau.encode_to(&mut state_blob);
        self.kappa.encode_to(&mut state_blob);
        self.lambda.encode_to(&mut state_blob);

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
            kappa: ValidatorsData::decode(state_blob)?,
            lambda: ValidatorsData::decode(state_blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputDisputes {
    Ok(OutputDataDisputes),
    Err(DisputesErrorCode),
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
            Ok(OutputDisputes::Ok(OutputDataDisputes::decode(output_blob)?))  
        } else if result == 1 {
            let error_type = output_blob.read_byte()?;
            let error = match error_type {
                0 => DisputesErrorCode::AlreadyJudged,
                1 => DisputesErrorCode::BadVoteSplit,
                2 => DisputesErrorCode::VerdictsNotSortedUnique,
                3 => DisputesErrorCode::JudgementsNotSortedUnique,
                4 => DisputesErrorCode::CulpritsNotSortedUnique,
                5 => DisputesErrorCode::FaultsNotSortedUnique,
                6 => DisputesErrorCode::NotEnoughCulprits,
                7 => DisputesErrorCode::NotEnoughFaults,
                8 => DisputesErrorCode::CulpritsVerdictNotBad,
                9 => DisputesErrorCode::FaultVerdictWrong,
                10 => DisputesErrorCode::OffenderAlreadyReported,
                11 => DisputesErrorCode::BadJudgementAge,
                12 => DisputesErrorCode::BadValidatorIndex,
                13 => DisputesErrorCode::BadSignature,
                14 => DisputesErrorCode::BadGuarantoorKey,
                15 => DisputesErrorCode::BadAuditorKey,
                16 => DisputesErrorCode::NoVerdictsFound,
                17 => DisputesErrorCode::AgesNotEqual,
                18 => DisputesErrorCode::CulpritKeyNotFound,
                19 => DisputesErrorCode::FaultKeyNotFound,
                _ => return Err(ReadError::InvalidData),
            };
            Ok(OutputDisputes::Err(error))
        } else {
            return Err(ReadError::InvalidData);
        }
    }
}