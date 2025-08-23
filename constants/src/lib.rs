pub mod node;
pub mod pvm;

pub const BUILD_PROFILE: &str = {
    #[cfg(feature = "tiny")]
    { "tiny" }
    #[cfg(feature = "full")]
    { "full" }
    #[cfg(not(any(feature = "tiny", feature = "full")))]
    { "unknown" }
};
