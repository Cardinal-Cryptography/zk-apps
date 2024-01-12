mod account;
mod note;
mod ops;
pub mod relations;
#[cfg(test)]
mod tests;
mod traits;

const USDT_TOKEN: [u8; 32] = [0x2_u8; 32];
