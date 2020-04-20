use std::fmt;
use std::error::Error;
use serde_json;

#[derive(Debug)]
pub struct RequestError {
    pub msg: String
}

impl fmt::Display for RequestError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "request error: {}", self.msg)
    }

}

impl Error for RequestError {
    fn description(&self) -> &str {
        "request error"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}


impl RequestError {

    pub fn new(msg: String) -> RequestError {
        RequestError {
            msg: msg
        }
    }

    pub fn str(msg: &str) -> RequestError {
        RequestError {
            msg: msg.to_string()
        }
    }

}

impl From<serde_json::error::Error> for RequestError {

   fn from(e: serde_json::error::Error) -> Self {
       RequestError::new(format!("parse error: {}", e))
   }

}
