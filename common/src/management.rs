use async_trait::async_trait;
use chrono::{DateTime, Utc};

use std::fmt::Display;

#[async_trait]
pub trait Manageable<'m, B>: Sync + Send {
    type Id;

    type Data;

    type Search;

    type Result;

    async fn create(
        &self,
        data: Self::Data,
        backend: &'m B,
    ) -> Result<Self::Data, Box<dyn crate::error::Error>>;
    async fn read(
        &self,
        search_opt: Self::Search,
        backend: &'m B,
    ) -> Result<Self::Result, Box<dyn crate::error::Error>>;
    async fn update(
        &self,
        id: Self::Id,
        data: Self::Data,
        backend: &'m B,
    ) -> Result<Self::Data, Box<dyn crate::error::Error>>;
    async fn delete(
        &self,
        id: Self::Id,
        backend: &'m B,
    ) -> Result<(), Box<dyn crate::error::Error>>;
}

#[async_trait]
pub trait Searchable: Sync + Send {
    type Id;
    fn get_id(&self) -> Option<Self::Id>;
    fn get_pattern(&self) -> Option<Box<dyn Display + Sync + Send>>;
    fn get_date_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)>;
    fn get_page(&self) -> usize;
    fn get_per_page(&self) -> usize;
}

#[async_trait]
pub trait SearchResult: Sync + Send {
    type Result;

    fn get_num_pages(&self) -> usize;

    fn get_result(&self) -> Box<dyn Iterator<Item = Self::Result>>;
}
