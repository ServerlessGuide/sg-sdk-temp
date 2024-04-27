use bevy_reflect::{reflect_trait, Reflect};
use std::any::Any;

use crate::HttpResult;

pub trait EnumConvert {
    fn enum_convert(f_name: &str, f_value: &str) -> HttpResult<(bool, Option<i32>)>;
}

pub trait ModelTrait: Any + Send + Sync + Sized + Reflect + Clone {
    fn clear_model(&self) -> Self
    where
        Self: Sized;
    fn set_field(&mut self, value: String, field_name: &str) -> Result<&Self, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized;
    fn new() -> Self
    where
        Self: Sized;
    fn clone_model(&self) -> Self
    where
        Self: Sized;
    fn get_field_str(&self, field_name: &str) -> Option<String>;
}

#[reflect_trait]
pub trait Validator {
    fn checkout(&self) -> std::result::Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

pub mod sg_err {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ResponseError {
        pub biz_res: String,
        pub message: Option<String>,
    }

    impl Display for ResponseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "response error in app",)
        }
    }

    impl std::error::Error for ResponseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(self)
        }

        fn description(&self) -> &str {
            "description() is deprecated; use Display"
        }

        fn cause(&self) -> Option<&dyn std::error::Error> {
            self.source()
        }
    }
}
