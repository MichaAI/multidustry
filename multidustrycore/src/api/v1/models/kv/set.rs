use bon::Builder;
use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Tabled, Builder, Serialize, Deserialize)]
pub struct SetRes {
    ok: bool,
}
