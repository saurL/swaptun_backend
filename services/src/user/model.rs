use serde::{Deserialize, Serialize};
use swaptun_models::UserColumn;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchField {
    Username,
    FirstName,
    LastName,
    Email,
}

impl SearchField {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchField::Username => "userName",
            SearchField::FirstName => "firstName",
            SearchField::LastName => "lastName",
            SearchField::Email => "email",
        }
    }
}

impl Into<UserColumn> for SearchField {
    fn into(self) -> UserColumn {
        match self {
            SearchField::Username => UserColumn::Username,
            SearchField::FirstName => UserColumn::FirstName,
            SearchField::LastName => UserColumn::LastName,
            SearchField::Email => UserColumn::Email,
        }
    }
}
