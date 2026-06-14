pub(crate) mod huff_sando_interface;

// `lil_router_interface` is sim-only: it depends on the simulator-facing lil
// router contract and is referenced exclusively from `simulator::lil_router`.
// Gate it behind the `simulate` feature so default-features builds don't compile
// the sim-only surface.
#[cfg(feature = "simulate")]
pub(crate) mod lil_router_interface;

#[cfg(test)]
mod tests;
