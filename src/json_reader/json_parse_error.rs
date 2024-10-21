#[derive(Debug)]
pub enum JsonParseError {
    CanNotFineStartOfTheJsonObject(String),
    CanNotFineStartOfTheArrayObject(String),
    Other(String),
}

impl JsonParseError {
    pub fn new(msg: String) -> Self {
        Self::Other(msg)
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::CanNotFineStartOfTheJsonObject(msg) => {
                return format!("Can not find start of the json object. {}", msg)
            }
            Self::CanNotFineStartOfTheArrayObject(msg) => {
                return format!("Can not find start of the array. {}", msg)
            }
            Self::Other(msg) => msg.to_string(),
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Self::CanNotFineStartOfTheJsonObject(msg) => msg,
            Self::CanNotFineStartOfTheArrayObject(msg) => msg,
            Self::Other(msg) => msg,
        }
    }
}
