#[derive(Debug)]
pub enum JsonParseError {
    CanNotFindStartOfTheJsonObject(String),
    CanNotFindStartOfTheArrayObject(String),
    Other(String),
}

impl JsonParseError {
    pub fn new(msg: String) -> Self {
        Self::Other(msg)
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::CanNotFindStartOfTheJsonObject(msg) => {
                return format!("Can not find start of the json object. {}", msg)
            }
            Self::CanNotFindStartOfTheArrayObject(msg) => {
                return format!("Can not find start of the array. {}", msg)
            }
            Self::Other(msg) => msg.to_string(),
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Self::CanNotFindStartOfTheJsonObject(msg) => msg,
            Self::CanNotFindStartOfTheArrayObject(msg) => msg,
            Self::Other(msg) => msg,
        }
    }
}
