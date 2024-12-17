#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum DataType {
    Integer,
    Decimal,
    Currency,
    Date,
    Email,
    Phone,
    Categorical,
    Text,
}
