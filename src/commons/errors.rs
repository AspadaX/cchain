#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error(
        "{number_missed_packages:?} required packages are missing in {chain_name}. Please install the following packages:{missed_packages}"
    )]
    MissingPackages {
        number_missed_packages: usize,
        missed_packages: String,
        chain_name: String,
    },
}
