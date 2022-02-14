use super::{MetricLabel, PortError, TelemetryError};
use crate::SharedString;
use either::{Either, Left, Right};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("{0}")]
    Port(#[from] PortError),

    #[error("{0}")]
    Telemetry(#[from] TelemetryError),

    #[error("failed to handle policy binding: {key} = {value}")]
    Binding { key: String, value: String },

    #[error("{0}")]
    Stage(#[from] anyhow::Error),
}

impl MetricLabel for GovernanceError {
    fn slug(&self) -> SharedString {
        "governance".into()
    }

    fn next(&self) -> Either<SharedString, Box<&dyn MetricLabel>> {
        match self {
            Self::Port(e) => Right(Box::new(e)),
            Self::Telemetry(e) => Right(Box::new(e)),
            Self::Binding { .. } => Left("binding".into()),
            Self::Stage(_) => Left("stage".into()),
        }
    }
}
