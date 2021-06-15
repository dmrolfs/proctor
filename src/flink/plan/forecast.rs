use crate::error::PlanError;
use crate::flink::MetricCatalog;

mod signal;
mod least_squares;
mod ridge_regression;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Workload {
    RecordsPerSecond(f64),
    NotEnoughData,
    HeuristicsExceedThreshold {},
}

pub trait WorkloadForecast {
    fn add_observation(&mut self, observation: MetricCatalog);
    fn clear(&mut self);
    fn predict_workload(&self) -> Result<Workload, PlanError>;
    fn workload_observation_from(metrics: MetricCatalog) -> (f64, f64) {
        (
            metrics.timestamp.timestamp() as f64,
            metrics.flow.task_nr_records_in_per_sec,
        )
    }
}
