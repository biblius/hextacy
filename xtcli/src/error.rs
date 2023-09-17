use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlxError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Yaml: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("Json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
