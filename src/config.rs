use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub modules_path: Option<String>,
    pub max_concurrent: Option<usize>,
}

impl Config {
    pub fn load() -> Result<Self, confy::ConfyError> {
        confy::load("dhd", None)
    }
}
