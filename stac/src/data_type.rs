use serde::{Deserialize, Serialize};

/// The data type gives information about the values in the file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// 8-bit integer
    Int8,

    /// 16-bit integer
    Int16,

    /// 32-bit integer
    Int32,

    /// 64-bit integer
    Int64,

    /// Unsigned 8-bit integer (common for 8-bit RGB PNG's)
    UInt8,

    /// Unsigned 16-bit integer
    UInt16,

    /// Unsigned 32-bit integer
    UInt32,

    /// Unsigned 64-bit integer
    UInt64,

    /// 16-bit float
    Float16,

    /// 32-bit float
    Float32,

    /// 64-bit float
    Float64,

    /// 16-bit complex integer
    CInt16,

    /// 32-bit complex integer
    CInt32,

    /// 32-bit complex float
    CFloat32,

    /// 64-bit complex float
    CFloat64,

    /// Other data type than the ones listed above (e.g. boolean, string, higher precision numbers)
    Other,
}
