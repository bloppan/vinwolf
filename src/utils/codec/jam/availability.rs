
use crate::types::{AvailabilityAssignment, AvailabilityAssignments, AvailabilityAssignmentsItem, WorkReport};
use crate::constants::CORES_COUNT;
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

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

impl Encode for AvailabilityAssignmentsItem {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

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

impl Encode for AvailabilityAssignments {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * CORES_COUNT);

        for assigment in self.0.iter() {
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

        let mut assignments: AvailabilityAssignments = AvailabilityAssignments{0: Box::new(std::array::from_fn(|_| None))};
        
        for assignment in assignments.0.iter_mut() {
            *assignment = AvailabilityAssignmentsItem::decode(blob)?;
        }

        Ok(assignments)
    }
}

