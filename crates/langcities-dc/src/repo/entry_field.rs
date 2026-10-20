use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::{EntryFieldAlias, EntryFieldRef, EntryRef, Id, Slug};
use langcities_server::dto::request::RequestContext;
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
    fn to_normal(self, usernames: &HashMap<Id, Slug>) -> Result<EntryFieldWithContext, DcAppError> {
        let username = usernames.get(&self.owner_auth_user_id.into());
        Ok(EntryFieldWithContext {
            entry_field: EntryFieldDto {
                id: self.id.into(),
                entry_id: self.entry_id.into(),
                slug: self.slug.parse().map_err(DcAppError::database)?,
                override_value: self.override_value,
                dirty_value: self.dirty_value,
                clean_value: self.clean_value,
                updated_at: self.updated_at,
            },
            entry_index: self.entry_index.into(),
            vernacular_id: self.vernacular_id.into(),
            vernacular_slug: self.vernacular_slug.parse().map_err(DcAppError::database)?,
            owner_auth_user_id: self.owner_auth_user_id.into(),
            owner_username: username.cloned(),
        })
    }
}

#[derive(Clone)]
struct FindManyEntryFieldComponents {
    pub exprs: HashMap<EntryFieldRef, Expr>,
    pub usernames: HashMap<Id, Slug>,
}

/// Intermediate data structure when finding entry fields by alias
///
/// Convert to [`FoundManyEntryFields`]
#[derive(Clone)]
struct FoundManyEntryFieldComponents {
    pub aliases: HashSet<EntryFieldRef>,
    pub entry_fields: Vec<EntryFieldWithSelectableContext>,
    pub usernames: HashMap<Id, Slug>,
    pub caller_id: Id,
    pub current_entry_id: Option<Id>,
}

impl FoundManyEntryFieldComponents {
    /// TODO: require original EntryFieldAlias to replace EntryFieldRef
    pub fn to_found_fields(self) -> Result<FoundManyEntryFields, DcAppError> {
        let entry_fields = self
            .entry_fields
            .into_iter()
            .map(|ef| ef.to_normal(&self.usernames))
            .collect::<Result<Vec<_>, _>>()?;
        let mut aliases = HashMap::<EntryFieldAlias, _>::new();
        'outer: for alias in self.aliases {
            for (i, entry_field) in entry_fields.iter().enumerate() {
                if entry_field.matches_ref(&alias, &self.caller_id, self.current_entry_id.as_ref())
                {
                    aliases.insert(alias.into(), Some(i));
                    continue 'outer;
                }
            }
            aliases.insert(alias.into(), None);
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
        alias: EntryFieldRef,
        context: RequestContext,
        action: EntryFieldAction,
        current_entry_id: Option<Id>,
    ) -> Result<Expr, DcAppError> {
        let (field_expr, entry_expr) = match &alias {
            EntryFieldRef::Id(id) => {
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
            EntryFieldRef::Qualified(alias) => {
                let entry_alias = alias
                    .entry
                    .clone()
                    .or_else(|| current_entry_id.map(|id| EntryRef::Id(id)))
                    .ok_or_else(|| {
                        DcAppError::bad_request("no entry specified, cannot search for field")
                    })?;
                let slug_expr = entry_fields::Column::Slug.eq(&*alias.slug);
                let entry_expr = self
                    .entry_repo
                    .generate_filter(entry_alias, context, action.to_entry_action())
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
        caller_id: Id,
        current_entry_id: Option<Id>,
    ) -> Result<FindManyEntryFieldComponents, DcAppError>
    where
        I: IntoIterator<Item = EntryFieldRef>,
    {
        let current_entry_ref = current_entry_id.clone().map(EntryRef::Id);
        let mut required_entries = HashSet::new();
        let aliases = aliases.into_iter().collect::<Vec<_>>();
        for alias in &aliases {
            let EntryFieldRef::Qualified(alias) = alias else {
                continue;
            };
            let entry_alias = alias
                .entry
                .clone()
                .or_else(|| current_entry_ref.clone())
                .ok_or_else(|| {
                    DcAppError::bad_request("no current entry specified for relative entry field")
                })?;
            required_entries.insert(entry_alias);
        }
        let entry_comps = self
            .entry_repo
            .generate_many_filter(required_entries, caller_id, current_entry_id)
            .await?;
        let mut exprs = HashMap::new();
        for alias in aliases {
            let expr = match &alias {
                EntryFieldRef::Id(id) => entry_fields::Column::Id.eq(**id),
                EntryFieldRef::Qualified(alias) => {
                    let slug_expr = entry_fields::Column::Slug.eq(&*alias.slug);
                    let entry_expr = entry_comps
                        .exprs
                        .get(alias.entry.as_ref().or(current_entry_ref.as_ref()).unwrap())
                        .unwrap()
                        .clone();
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
        alias: EntryFieldRef,
        context: RequestContext,
        action: EntryFieldAction,
        current_entry_id: Option<Id>,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expr = self
            .generate_filter(alias, context, action, current_entry_id)
            .await?;
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
        current_entry_id: Option<Id>,
    ) -> Result<FoundManyEntryFieldComponents, DcAppError>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = EntryFieldRef>,
    {
        let ef_comps = self
            .generate_many_filter(aliases, caller_id.clone(), current_entry_id.clone())
            .await?;
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
            current_entry_id,
        })
    }

    pub async fn get_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldRef,
        context: RequestContext,
        current_entry_id: Option<Id>,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.inner_get_entry_field(
            conn,
            alias,
            context,
            EntryFieldAction::Read,
            current_entry_id,
        )
        .await
    }

    pub async fn get_entry_fields<C, I>(
        &self,
        conn: &C,
        aliases: I,
        context: RequestContext,
        current_entry_id: Option<Id>,
    ) -> Result<FoundManyEntryFields, DcAppError>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = EntryFieldRef>,
    {
        let Some(caller_id) = context.caller_id else {
            return Err(DcAppError::bad_request("caller_id cannot be None"));
        };
        let components = self
            .inner_find_many_entry_fields(conn, aliases, caller_id, current_entry_id)
            .await?;
        components.to_found_fields()
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
        let entry_ref: EntryRef = dto.entry.0.clone().into();
        let entry = self
            .entry_repo
            .inner_get_entry(conn, entry_ref, context, EntryAction::Write)
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
        alias: EntryFieldRef,
        dto: UpdateEntryFieldDto,
        context: RequestContext,
        current_entry_id: Option<Id>,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.update_entry_field_returning_original(conn, alias, dto, context, current_entry_id)
            .await
            .map(|o| o.map(|(_old, new)| new))
    }

    pub async fn update_entry_field_returning_original<C>(
        &self,
        conn: &C,
        alias: EntryFieldRef,
        dto: UpdateEntryFieldDto,
        context: RequestContext,
        current_entry_id: Option<Id>,
    ) -> Result<Option<(entry_fields::Model, entry_fields::Model)>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let old = match self
            .inner_get_entry_field(
                conn,
                alias,
                context,
                EntryFieldAction::Write,
                current_entry_id,
            )
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
        alias: EntryFieldRef,
        context: RequestContext,
        current_entry_id: Option<Id>,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry_field = match self
            .inner_get_entry_field(
                conn,
                alias,
                context,
                EntryFieldAction::Delete,
                current_entry_id,
            )
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
