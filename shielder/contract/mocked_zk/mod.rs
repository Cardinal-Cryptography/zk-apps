pub mod account;
pub mod note;
pub mod ops;
pub mod relations;
#[cfg(test)]
mod tests;
pub mod traits;

const USDT_TOKEN: [u8; 32] = [0x2_u8; 32];
