use erased_serde::serialize_trait_object;
use std::fmt::Display;
pub trait Userable: erased_serde::Serialize + Sync + Send {
    fn get_id(&self) -> Box<dyn Display + Sync + Send>;
    fn get_first_name(&self) -> String;
    fn get_lastname(&self) -> String;
    fn get_email(&self) -> String;
}

serialize_trait_object!(Userable);
