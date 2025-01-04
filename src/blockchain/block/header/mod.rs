
// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, 
// winning-tickets and offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; 
// the entropy-yielding, vrf signature, and a block seal. Excepting the Genesis header, all block headers H have
// an associated parent header, whose hash is Hp.

// The epoch and winning-tickets markers are information placed in the header in order to minimize 
// data transfer necessary to determine the validator keys associated with any given epoch. They 
// are particularly useful to nodes which do not synchronize the entire state for any given block 
// since they facilitate the secure tracking of changes to the validator key sets using only the 
// chain of headers.


// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket 
// contest does not complete adequately (a very much unexpected eventuality).The epoch marker is
// either empty or, if the block is the first in a new epoch, then a tuple of the epoch randomness 
// and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in 
// the next epoch.



// The Tickets Marker provides the series of EPOCH_LENGTH (600) slot sealing “tickets” for the next epoch. Is either 
// empty or, if the block is the first after the end of the submission period for tickets and if the ticket accumulator 
// is saturated, then the final sequence of ticket identifiers.

