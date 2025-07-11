
use crate::jam_types::{AvailabilityAssignment, AvailabilityAssignments, AvailabilityAssignmentsItem, WorkReport};
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

impl Encode for AvailabilityAssignments {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * CORES_COUNT);

        for assigment in self.list.iter() {
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

        let mut assignments: AvailabilityAssignments = AvailabilityAssignments::default();
        
        for assignment in assignments.list.iter_mut() {
            *assignment = AvailabilityAssignmentsItem::decode(blob)?;
        }

        Ok(assignments)
    }
}

