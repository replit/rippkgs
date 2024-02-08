use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "package")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub attribute: String,
  pub store_path: String,
  pub name: String,
  pub version: String,
  pub description: Option<String>,
  pub homepage: Option<String>,
  pub long_description: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

