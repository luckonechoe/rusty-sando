pub(crate) mod huff_sando_interface;

// `lil_router_interface` is sim-only: it depends on cfmms + ethers and is
// referenced exclusively from `simulator::lil_router`. Gate it behind the
// `simulate` feature so default-features builds don't pull legacy deps.
#[cfg(feature = "simulate")]
pub(crate) mod lil_router_interface;
