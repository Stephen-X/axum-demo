use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Value {
    pub value: String,
}
