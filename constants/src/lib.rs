pub mod node;
pub mod pvm;

pub const BUILD_PROFILE: &str = {
    #[cfg(all(feature = "tiny", not(feature = "full")))]
    { "tiny" }
    #[cfg(feature = "full")]
    { "full" }
    #[cfg(not(any(feature = "tiny", feature = "full")))]
    { "unknown" }
};
