use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::{EntryFieldAlias, Id};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait,
    FromQueryResult, IntoActiveModel, JoinType, QueryFilter, QuerySelect, RelationTrait,
    TransactionError, TransactionTrait,
    sea_query::{Expr, Query},
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::entry_fields::{
        CreateEntryFieldDto, EntryFieldDto, EntryFieldWithContext, FoundManyEntryFields,
        UpdateEntryFieldDto,
    },
    entity::{dc_users, entries, entry_fields, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    repo::{
        entry::{EntryAction, EntryRepo},
        vernacular::{VernacularAction, VernacularRepo},
    },
    state::AppState,
};

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
struct EntryFieldWithSelectableContext {
    pub id: i64,
    pub entry_id: i64,
    pub slug: String,
    pub override_value: String,
    pub dirty_value: String,
    pub clean_value: String,
    pub updated_at: DateTime<Utc>,
    pub entry_index: i64,
    pub vernacular_id: i64,
    pub vernacular_slug: String,
    pub owner_auth_user_id: i64,
    // pub owner_username: String,
}

impl EntryFieldWithSelectableContext {
    fn to_normal(self, usernames: &HashMap<i64, String>) -> EntryFieldWithContext {
        let username = usernames.get(&self.owner_auth_user_id);
        EntryFieldWithContext {
            entry_field: EntryFieldDto {
                id: self.id,
                entry_id: self.entry_id,
                slug: self.slug,
                override_value: self.override_value,
                dirty_value: self.dirty_value,
                clean_value: self.clean_value,
                updated_at: self.updated_at,
            },
            entry_index: self.entry_index,
            vernacular_id: self.vernacular_id,
            vernacular_slug: self.vernacular_slug,
            owner_auth_user_id: self.owner_auth_user_id,
            owner_username: username.cloned(),
        }
    }
}

#[derive(Clone)]
struct FindManyEntryFieldComponents {
    pub exprs: HashMap<EntryFieldAlias, Expr>,
    pub usernames: HashMap<i64, String>,
}

/// Intermediate data structure when finding entry fields by alias
///
/// Convert to [`FoundManyEntryFields`]
#[derive(Clone)]
struct FoundManyEntryFieldComponents {
    pub aliases: HashSet<EntryFieldAlias>,
    pub entry_fields: Vec<EntryFieldWithSelectableContext>,
    pub usernames: HashMap<i64, String>,
    pub caller_id: Id,
}

impl TryInto<FoundManyEntryFields> for FoundManyEntryFieldComponents {
    type Error = DcAppError;

    fn try_into(self) -> Result<FoundManyEntryFields, DcAppError> {
        let entry_fields = self
            .entry_fields
            .into_iter()
            .map(|ef| ef.to_normal(&self.usernames))
            .collect::<Vec<_>>();
        let mut aliases = HashMap::new();
        'outer: for alias in self.aliases {
            for (i, entry_field) in entry_fields.iter().enumerate() {
                if entry_field.matches_alias(&alias, &self.caller_id) {
                    aliases.insert(alias, Some(i));
                    continue 'outer;
                }
            }
            aliases.insert(alias, None);
        }
        Ok(FoundManyEntryFields {
            entry_fields,
            aliases,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntryFieldAction {
    Read,
    Write,
    Delete,
}

impl EntryFieldAction {
    pub fn enforce_owner(&self) -> bool {
        !matches!(self, Self::Read)
    }

    pub fn to_entry_action(&self) -> EntryAction {
        match self {
            Self::Read => EntryAction::Read,
            Self::Write | Self::Delete => EntryAction::Write,
        }
    }

    pub fn to_vernacular_action(&self) -> VernacularAction {
        match self {
            Self::Read => VernacularAction::Read,
            Self::Write | Self::Delete => VernacularAction::Write,
        }
    }
}

#[derive(Clone)]
pub struct EntryFieldRepo {
    pub entry_repo: EntryRepo,
}

impl EntryFieldRepo {
    pub fn new(entry_repo: impl Into<EntryRepo>) -> Self {
        let entry_repo = entry_repo.into();
        Self { entry_repo }
    }

    pub fn from_state(state: impl Into<AppState>) -> Self {
        Self::new(EntryRepo::from_state(state))
    }

    #[inline]
    pub fn state(&self) -> &AppState {
        self.entry_repo.state()
    }

    #[inline]
    pub fn vernacular_repo(&self) -> &VernacularRepo {
        &self.entry_repo.vernacular_repo
    }

    pub async fn generate_filter(
        &self,
        alias: EntryFieldAlias,
        context: RequestContext,
        action: EntryFieldAction,
    ) -> Result<Expr, DcAppError> {
        let (field_expr, entry_expr) = match &alias {
            EntryFieldAlias::Id(id) => {
                let id_expr = entry_fields::Column::Id.eq(**id);
                if action.enforce_owner() {
                    if let Some(auth_user_id) = &context.caller_id {
                        let ver_expr = VernacularRepo::generate_owner_filter(**auth_user_id);
                        let mut entry_subquery = entries::Entity::find()
                            .left_join(vernaculars::Entity)
                            .filter(ver_expr)
                            .select_only()
                            .column(entries::Column::Id);
                        let entry_expr = entry_fields::Column::EntryId
                            .in_subquery(QuerySelect::query(&mut entry_subquery).to_owned());
                        (id_expr, Some(entry_expr))
                    } else {
                        return Err(DcAppError::unauthorized("unauthorized"));
                    }
                } else {
                    (id_expr, None)
                }
            }
            EntryFieldAlias::Alias(alias) => {
                let slug_expr = entry_fields::Column::Slug.eq(&*alias.slug);
                let entry_expr = self
                    .entry_repo
                    .generate_filter(alias.entry_alias.clone(), context, action.to_entry_action())
                    .await?;
                (slug_expr, Some(entry_expr))
            }
        };
        let expr = if let Some(entry_expr) = entry_expr {
            entry_fields::Column::EntryId
                .in_subquery(
                    Query::select()
                        .from(entries::Entity)
                        .column(entries::Column::Id)
                        .and_where(entry_expr)
                        .to_owned(),
                )
                .and(field_expr)
        } else {
            field_expr
        };
        Ok(expr)
    }

    /// assumes join between entry_fields, entries, vernaculars and dc_users
    pub(super) async fn generate_many_filter<I>(
        &self,
        aliases: I,
        caller_id: i64,
    ) -> Result<FindManyEntryFieldComponents, DcAppError>
    where
        I: IntoIterator<Item = EntryFieldAlias>,
    {
        let mut required_entries = HashSet::new();
        let aliases = aliases.into_iter().collect::<Vec<_>>();
        for alias in &aliases {
            let EntryFieldAlias::Alias(alias) = alias else {
                continue;
            };
            required_entries.insert(alias.entry_alias.clone());
        }
        let entry_comps = self
            .entry_repo
            .generate_many_filter(required_entries, caller_id)
            .await?;
        let mut exprs = HashMap::new();
        for alias in aliases {
            let expr = match &alias {
                EntryFieldAlias::Id(id) => entry_fields::Column::Id.eq(**id),
                EntryFieldAlias::Alias(alias) => {
                    let slug_expr = entry_fields::Column::Slug.eq(&*alias.slug);
                    let entry_expr = entry_comps.exprs.get(&alias.entry_alias).unwrap().clone();
                    slug_expr.and(entry_expr)
                }
            };
            exprs.insert(alias, expr);
        }
        Ok(FindManyEntryFieldComponents {
            exprs,
            usernames: entry_comps.usernames,
        })
    }

    async fn inner_get_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
        action: EntryFieldAction,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expr = self.generate_filter(alias, context, action).await?;
        entry_fields::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    async fn inner_find_many_entry_fields<C, I>(
        &self,
        conn: &C,
        aliases: I,
        caller_id: Id,
    ) -> Result<FoundManyEntryFieldComponents, DcAppError>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = EntryFieldAlias>,
    {
        let ef_comps = self.generate_many_filter(aliases, *caller_id).await?;
        let mut aliases = HashSet::new();
        let mut cond = Condition::any();
        for (alias, expr) in ef_comps.exprs {
            cond = cond.add(expr);
            aliases.insert(alias);
        }
        let entry_fields = entry_fields::Entity::find()
            .column_as(entries::Column::Index, "entry_index")
            .column_as(vernaculars::Column::Id, "vernacular_id")
            .column_as(vernaculars::Column::Slug, "vernacular_slug")
            .column_as(dc_users::Column::AuthUserId, "owner_auth_user_id")
            .filter(cond)
            .left_join(entries::Entity)
            .join(JoinType::LeftJoin, entries::Relation::Vernaculars.def())
            .join(JoinType::LeftJoin, vernaculars::Relation::DcUsers.def())
            .into_model::<EntryFieldWithSelectableContext>()
            .all(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(FoundManyEntryFieldComponents {
            aliases,
            entry_fields,
            usernames: ef_comps.usernames,
            caller_id,
        })
    }

    pub async fn get_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.inner_get_entry_field(conn, alias, context, EntryFieldAction::Read)
            .await
    }

    pub async fn get_entry_fields<C, I>(
        &self,
        conn: &C,
        aliases: I,
        context: RequestContext,
    ) -> Result<FoundManyEntryFields, DcAppError>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = EntryFieldAlias>,
    {
        let Some(caller_id) = context.caller_id else {
            return Err(DcAppError::bad_request("caller_id cannot be None"));
        };
        let components = self
            .inner_find_many_entry_fields(conn, aliases, caller_id)
            .await?;
        components.try_into()
    }

    async fn inner_create_entry_field<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldDto,
        context: RequestContext,
    ) -> Result<entry_fields::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry = self
            .entry_repo
            .inner_get_entry(conn, dto.entry.0.clone(), context, EntryAction::Write)
            .await?
            .ok_or_else(|| DcAppError::not_found(format!("entry {} not found", dto.entry)))?;
        let entry_field = entry_fields::Entity::insert(dto.to_active_model(&entry))
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        // TODO: add job
        Ok(entry_field)
    }

    pub async fn create_entry_field<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldDto,
        context: RequestContext,
    ) -> Result<entry_fields::Model, DcAppError>
    where
        C: TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_create_entry_field(txn, dto, context).await })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }

    pub async fn update_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        dto: UpdateEntryFieldDto,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.update_entry_field_returning_original(conn, alias, dto, context)
            .await
            .map(|o| o.map(|(_old, new)| new))
    }

    pub async fn update_entry_field_returning_original<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        dto: UpdateEntryFieldDto,
        context: RequestContext,
    ) -> Result<Option<(entry_fields::Model, entry_fields::Model)>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let old = match self
            .inner_get_entry_field(conn, alias, context, EntryFieldAction::Write)
            .await?
        {
            Some(model) => model,
            None => return Ok(None),
        };
        let mut entry_field = old.clone().into_active_model();
        dto.update_active_model(&mut entry_field);
        let new = entry_field
            .update(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(Some((old, new)))
    }

    pub async fn delete_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry_field = match self
            .inner_get_entry_field(conn, alias, context, EntryFieldAction::Delete)
            .await?
        {
            Some(model) => model,
            None => return Ok(None),
        };
        entry_fields::Entity::delete_by_id(entry_field.id)
            .exec(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(Some(entry_field))
        // TODO: trigger jobs
    }
}
