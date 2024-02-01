use common::error::CoreError;
use common::error::CoreError::{DataError, ResourceNotFound, UnkownError};
use sea_orm::DbErr;
use std::convert::From;
use uuid::Error;

#[derive(Debug)]
pub struct AsCoreError(CoreError);

impl common::error::Error for AsCoreError {
    fn get_core_error(&self) -> CoreError {
        self.0.to_owned()
    }
}

impl From<DbErr> for AsCoreError {
    fn from(value: DbErr) -> Self {
        if let Some(sql_error) = value.sql_err() {
            AsCoreError(DataError(sql_error.to_string()))
        } else {
            match value {
                DbErr::RecordNotFound(s) => AsCoreError(ResourceNotFound(s)),
                unknown => AsCoreError(UnkownError(unknown.to_string())),
            }
        }
    }
}

impl From<uuid::Error> for AsCoreError {
    fn from(value: Error) -> Self {
        AsCoreError(DataError(value.to_string()))
    }
}
