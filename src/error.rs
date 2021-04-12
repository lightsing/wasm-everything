
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ser/de error {0}")]
    Bincode(#[from] bincode::Error)
}