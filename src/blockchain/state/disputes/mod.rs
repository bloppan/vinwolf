// Jam provides a means of recording judgments: consequential votes amongst most of the validators over the
// validity of a work-report (a unit of work done within Jam). Such collections of judgments are known as 
// verdicts. Jam also provides a means of registering offenses, judgments and guarantees which dissent with an
// established verdict. Together these form the disputes system.

// The registration of a verdict is not expected to happen very often in practice, however it is an important 
// security backstop for removing and banning invalid work-reports from the processing pipeline as well as 
// removing troublesome keys from the validator set where there is consensus over their malfunction. It also 
// helps coordinate nodes to revert chain-extensions containing invalid work-reports and provides a convenient 
// means of aggregating all offending validators for punishment in a higher-level system.

// Judgement statements come about naturally as part of the auditing process and are expected to be positive,
// further affirming the guarantors' assertion that the workreport is valid. In the event of a negative judgment, 
// then all validators audit said work-report and we assume a verdict will be reached. Auditing and guaranteeing 
// are offchain processes. 

// A judgment against a report implies that the chain is already reverted to some point prior to the accumulation 
// of said report, usually forking at the block immediately prior to that at which accumulation happened. Authoring 
// a block with a non-positive verdict has the effect of cancelling its imminent accumulation.

// Registering a verdict also has the effect of placing a permanent record of the event on-chain and allowing any
// offending keys to be placed on-chain both immediately or in forthcoming blocks, again for permanent record.

// Having a persistent on-chain record of misbehavior is helpful in a number of ways. It provides a very simple
// means of recognizing the circumstances under which action against a validator must be taken by any higher-level
// validator-selection logic. Should Jam be used for a public network such as Polkadot, this would imply the slashing 
// of the offending validator's stake on the staking parachain.

// As mentioned, recording reports found to have a high confidence of invalidity is important to ensure that said
// reports are not allowed to be resubmitted. Conversely, recording reports found to be valid ensures that additional
// disputes cannot be raised in the future of the chain.


use crate::jam_types::{
    AvailabilityAssignments, DisputesExtrinsic, DisputesRecords, Offenders, OutputDataDisputes, WorkReportHash,DisputesErrorCode 
};
use crate::utils::common::has_duplicates;
use crate::blockchain::state::ProcessError;

pub fn process(
    disputes_state: &mut DisputesRecords, 
    availability_state: &mut AvailabilityAssignments,
    disputes_extrinsic: &DisputesExtrinsic
) -> Result<OutputDataDisputes, ProcessError> {

    let output_data = disputes_extrinsic.process(disputes_state, availability_state)?;

    Ok(OutputDataDisputes { 
        offenders_mark: output_data.offenders_mark 
    })
}


impl DisputesRecords {

    pub fn update(
        &mut self, 
        new_wr_reported: &DisputesRecords, 
        culprits_keys: &[WorkReportHash],
        faults_keys: &[WorkReportHash]) 
    -> Result<Offenders, ProcessError> {

        let new_offenders = Vec::from([culprits_keys, faults_keys].concat());

        // In the disputes extrinsic can not be offenders already reported
        let all_offenders = Vec::from([self.offenders.clone(), new_offenders.clone()].concat());
        if has_duplicates(&all_offenders) {
            return Err(ProcessError::DisputesError(DisputesErrorCode::OffenderAlreadyReported));
        }   

        // If the state was initialized, then we save the auxiliar records in the state
        self.good.extend_from_slice(&new_wr_reported.good);
        self.bad.extend_from_slice(&new_wr_reported.bad);
        self.wonky.extend_from_slice(&new_wr_reported.wonky);
        let mut offenders = new_offenders.clone();
        offenders.sort();
        self.offenders.extend_from_slice(&offenders);

        Ok(new_offenders)
    }
}