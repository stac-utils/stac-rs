use serde::{Deserialize, Serialize};

/// To support "generic" clients that want to access multiple OGC API Features
/// implementations - and not "just" a specific API / server, the server has to
/// declare the conformance classes it implements and conforms to.
#[derive(Debug, Serialize, Deserialize)]
pub struct Conformance {
    /// The conformance classes it implements and conforms to.
    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}
