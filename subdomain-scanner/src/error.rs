use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Usage: subdomain-scanner <adress.com>")]
    CliUsage,
}
