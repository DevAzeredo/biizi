use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum JobStatus {
    OPEN,
    PENDING,
    COMPLETED,
    CANCELLED,
}
