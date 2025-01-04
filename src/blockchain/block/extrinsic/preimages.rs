// Preimages are static data which is presently being requested to be available for workloads to be able to 
// fetch on demand. Prior to accumulation, we must first integrate all preimages provided in the lookup extrinsic. 
// The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and without 
// duplicates. The data must have been solicited by a service but not yet be provided.

