use std::fmt::Display;

use crate::error::AsCoreError;
use async_trait::async_trait;
use common::{
    management::{Manageable, SearchResult, Searchable},
    user::userable::Userable,
};
use entity::user::{ActiveModel, Column, Entity, Model};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, Paginator, PaginatorTrait,
    QueryFilter, SelectModel,
};
use sea_orm::{ColumnTrait, ConnectionTrait};
use uuid::Uuid;
pub struct UserManagement;
pub struct UserSearchResult {
    num_pages: usize,
    result: Vec<Model>,
}

#[async_trait]
impl<'a, B: ConnectionTrait> Manageable<'a, B> for UserManagement {
    type Id = Box<dyn Display + Sync + Send>;
    type Data = Box<dyn Userable>;
    type Search = Box<dyn Searchable<Id = Box<dyn Display + Sync + Send>>>;
    type Result = Box<dyn SearchResult<Result = Box<dyn Userable>>>;

    async fn create(
        &self,
        data: Self::Data,
        backend: &'a B,
    ) -> Result<Self::Data, Box<dyn common::error::Error>> {
        Ok(Box::new(
            ActiveModel {
                id: Set(Uuid::new_v4()),
                first_name: Set((*data).get_first_name()),
                last_name: Set((*data).get_lastname()),
                email: Set((*data).get_email()),
                ..Default::default()
            }
            .insert(backend)
            .await
            .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?,
        ))
    }

    async fn read(
        &self,
        search_opt: Self::Search,
        backend: &'a B,
    ) -> Result<Self::Result, Box<dyn common::error::Error>> {
        let mut select_users;
        if let Some(id) = search_opt.get_id() {
            select_users =
                Entity::find_by_id(id.to_string().as_str().parse::<Uuid>().map_err(|e| {
                    Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>
                })?);
        } else {
            select_users = Entity::find();
        };
        if let Some(pattern) = search_opt.get_pattern() {
            select_users = select_users.filter(
                Column::FirstName
                    .eq(pattern.to_string())
                    .or(Column::LastName.eq(pattern.to_string()))
                    .or(Column::Email.eq(pattern.to_string())),
            );
        };
        let paginator: Paginator<_, SelectModel<Model>> = select_users.paginate(
            backend,
            TryInto::<u64>::try_into(search_opt.get_per_page()).unwrap(),
        );
        let result = Box::new(UserSearchResult {
            num_pages: paginator
                .num_pages()
                .await
                .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?
                as usize,
            result: paginator
                .fetch_page(TryInto::<u64>::try_into(search_opt.get_page()).unwrap() - 1)
                .await
                .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?,
        });
        Ok(result as Self::Result)
    }

    async fn update(
        &self,
        id: Self::Id,
        data: Self::Data,
        backend: &'a B,
    ) -> Result<Self::Data, Box<dyn common::error::Error>> {
        if let Some(selected_model) = Entity::find_by_id(
            id.to_string()
                .as_str()
                .parse::<Uuid>()
                .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?,
        )
        .one(backend)
        .await
        .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?
        {
            let mut active_model = selected_model.into_active_model();
            active_model.first_name = Set(data.get_first_name());
            active_model.last_name = Set(data.get_lastname());
            active_model.email = Set(data.get_email());
            return Ok(Box::new(
                active_model
                    .update(backend)
                    .await
                    .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?,
            ) as Self::Data);
        } else {
            return Err(Box::new(AsCoreError::from(DbErr::RecordNotFound(
                String::from("Record not found !"),
            ))));
        }
    }

    async fn delete(
        &self,
        id: Self::Id,
        backend: &'a B,
    ) -> Result<(), Box<dyn common::error::Error>> {
        if let Some(selected_model) = Entity::find_by_id(
            id.to_string()
                .as_str()
                .parse::<Uuid>()
                .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?,
        )
        .one(backend)
        .await
        .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>)?
        {
            return selected_model
                .delete(backend)
                .await
                .map(|_| ())
                .map_err(|e| Box::new(AsCoreError::from(e)) as Box<dyn common::error::Error>);
        } else {
            return Err(Box::new(AsCoreError::from(DbErr::RecordNotFound(
                String::from("Record not found !"),
            ))));
        }
    }
}

impl SearchResult for UserSearchResult {
    type Result = Box<dyn Userable>;

    fn get_num_pages(&self) -> usize {
        self.num_pages
    }

    fn get_result(&self) -> Box<dyn Iterator<Item = Self::Result>> {
        Box::new(
            self.result
                .clone()
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn Userable>),
        )
    }
}
