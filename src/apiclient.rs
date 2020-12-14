use crate::{types::*, Error, UserCache};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};

const DEFAULT_ENDPOINT: &str = "https://zenkit.com/api/v1";
const API_TOKEN_ENV_VAR: &str = "ZENKIT_API_TOKEN";
/// Zenkit http/API client
#[derive(Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    url_prefix: String, // url prefix
    ratelimit: Option<u32>,
    ratelimit_remaining: Option<u32>,
    /// cache of workspaces, loaded with get_all_workspaces_and_ids
    workspaces: RwLock<Vec<Arc<WorkspaceData>>>,
    /// cache of lists
    lists: RwLock<Vec<Arc<ListInfo>>>,
}

/// Initialization parameters for Zenkit Api client
pub struct ApiConfig {
    /// Secret API Token. Default value is from environment: ZENKIT_API_TOKEN
    pub token: String,
    /// HTTPS endpoint. Defaults to "https://zenkit.com/api/v1"
    pub endpoint: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            endpoint: String::from(DEFAULT_ENDPOINT),
            token: std::env::var(API_TOKEN_ENV_VAR).ok().unwrap_or_default(),
        }
    }
}

// generate user-agent header value from package version in the form "zenkit rs x.y.z"
fn user_agent_header() -> HeaderValue {
    let agent = &format!(
        "{} rs {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    HeaderValue::from_str(agent).unwrap_or_else(|_| HeaderValue::from_static("zenkit_rust"))
}

impl ApiClient {
    /// Constructs a new ApiClient.
    /// Because there are some functions that use the "global" get_api(),
    /// most users should use [init_api] instead of this constructor.
    /// Error if token is non-ascii
    pub(crate) fn new(config: ApiConfig) -> Result<Self, Error> {
        use reqwest::header::{CONTENT_TYPE, USER_AGENT};
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, user_agent_header());
        headers.insert(
            "Zenkit-API-Key",
            HeaderValue::from_str(&config.token)
                .map_err(|_| Error::Other("token has non-ascii chars".to_string()))?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            url_prefix: config.endpoint,
            ratelimit: None,
            ratelimit_remaining: None,
            workspaces: RwLock::new(Vec::new()),
            lists: RwLock::new(Vec::new()),
        })
    }

    /// Returns the rate limit returned on the most recent api call
    /// Not yet implemented
    pub fn get_rate_limit(&self) -> Option<u32> {
        self.ratelimit
    }

    /// Returns the rate limit remaining on the most recent api call
    /// Not yet implemented
    pub fn get_rate_limit_remaining(&self) -> Option<u32> {
        self.ratelimit_remaining
    }

    /// Check response for http errors and deserialize to requested object type.
    /// This is called on every response returned from the api client
    async fn json<T: DeserializeOwned>(&self, resp: Response) -> Result<T, Error> {
        //
        // see if api gave us pushback for rate limit
        //   if so, store them internally; otherwise, set None for those fields
        // Since self is not mut, we would need to store these in a Cell
        // match resp.headers.get("x-ratelimit-limit")
        // match resp.headers.get("x-ratelimit-remaining")
        // match resp.headers.get("x-ratelimit-reset") // time when api ok to use again
        //
        let status = &resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            // attempt to parse response as Zenkit error
            if let Ok(err_res) = serde_json::from_slice::<ErrorResult>(&bytes) {
                return Err(Error::ApiError(status.as_u16(), Some(err_res.error)));
            }
            return Err(Error::Other(format!(
                "Server returned status {}:{}",
                status.as_u16(),
                String::from_utf8_lossy(bytes.as_ref())
            )));
        }
        match serde_json::from_slice(&bytes) {
            Ok(obj) => Ok(obj),
            Err(e) => {
                eprintln!(
                    "Error deserializing result {}. data:\n{}",
                    e.to_string(),
                    String::from_utf8_lossy(&bytes)
                );
                Err(e.into())
            }
        }
    }

    /// Returns users in workspace. This method caches the user list so subsequent
    /// calls for the same workspace use the in-memory list.
    pub async fn get_users(&self, workspace_id: ID) -> Result<Vec<Arc<User>>, Error> {
        let ws_cache_read = self.workspaces.read()?;
        let wd = match ws_cache_read
            .iter()
            .find(|w| w.workspace.id == workspace_id)
        {
            Some(w) => w,
            None => {
                return Err(Error::Other(format!(
                    "get_workspace_users: invalid workspace_id '{}'",
                    workspace_id
                )))
            }
        };
        let wd2 = wd.clone();
        // drop to avoid holding lock across await. Cost is a very small chance of duplicate
        // fetches but gain is preventing delay of other uses of workspace list.
        drop(ws_cache_read);
        Ok(wd2.users().await?)
    }

    /// Returns users in the workspace. Bypasses cache and uses zenkit api directly.
    /// See also get_users.
    pub async fn get_users_raw(&self, workspace_id: ID) -> Result<Vec<User>, Error> {
        let url = format!("{}/workspaces/{}/users", self.url_prefix, workspace_id);
        let resp = self.client.get(&url).send().await?;
        self.json(resp).await
    }

    /// Find first user matching predicate
    pub async fn find_user<P>(
        &self,
        workspace_id: ID,
        predicate: P,
    ) -> Result<Option<Arc<User>>, Error>
    where
        P: Fn(&Arc<User>) -> bool,
    {
        let ws_cache_read = self.workspaces.read()?;
        let wd = match ws_cache_read
            .iter()
            .find(|w| w.workspace.id == workspace_id)
        {
            Some(w) => w,
            None => {
                return Err(Error::Other(format!(
                    "get_workspace_users: invalid workspace_id '{}'",
                    workspace_id
                )))
            }
        };
        let wd2 = wd.clone();
        drop(ws_cache_read);
        Ok(wd2.find_user(predicate).await?)
    }

    /// Finds the user id for the name. Name parameter can be display name, full name, or uuid.
    /// String matching is case-insensitive. Return value is Some(id) if found,
    /// None if no match, or Err if there was a network problem getting the user list.
    pub async fn get_user_id(&self, workspace_id: ID, name: &str) -> Result<Option<ID>, Error> {
        let lc_name = name.to_lowercase();
        let id = self
            .find_user(workspace_id, |u| {
                u.display_name.to_lowercase() == lc_name
                    || u.full_name.to_lowercase() == lc_name
                    || u.uuid == lc_name
            })
            .await?
            .map(|u| u.id);
        Ok(id)
    }

    /// get accesses for the user
    pub async fn get_user_accesses(&self) -> Result<Vec<Access>, Error> {
        let resp = self
            .client
            .get(&format!("{}/users/me/access", self.url_prefix))
            .send()
            .await?;
        self.json(resp).await
    }

    /// Returns shared accesses for user
    pub async fn get_shared_accesses<A: Into<AllId>>(
        &self,
        user_allid: A,
    ) -> Result<SharedAccesses, Error> {
        let url = format!(
            "{}/users/me/matching-access/{}",
            self.url_prefix,
            user_allid.into()
        );
        let resp = self.client.get(&url).send().await?;
        self.json(resp).await
    }

    /// returns schema fields of list
    pub async fn get_list_elements<A: Into<AllId>>(
        &self,
        list_allid: A,
    ) -> Result<Vec<Element>, Error> {
        let url = format!("{}/lists/{}/elements", self.url_prefix, list_allid.into());
        let resp = self.client.get(&url).send().await?;
        self.json(resp).await
    }

    /// Returns a single list item
    pub async fn get_entry<L: Into<AllId>, E: Into<AllId>>(
        &self,
        list_allid: L,
        entry_allid: E,
    ) -> Result<Entry, Error> {
        let url = format!(
            "{}/lists/{}/entries/{}",
            self.url_prefix,
            list_allid.into(),
            entry_allid.into()
        );
        let resp = self.client.get(&url).send().await?;
        self.json(resp).await
    }

    /// Returns items from list (possibly filtered/sorted), with pagination
    /// Parameter may be id or uuid. To use name lookup, use get_list_info.
    /// See also get_list_entries_for_view
    pub async fn get_list_entries<A: Into<AllId>>(
        &self,
        list_allid: A,
        params: &GetEntriesRequest,
    ) -> Result<Vec<Entry>, Error> {
        let url = format!(
            "{}/lists/{}/entries/filter",
            self.url_prefix,
            list_allid.into()
        );
        let resp = self.client.post(&url).json(&params).send().await?;
        self.json(resp).await
    }

    /// Returns list items sorted by last update (asc or desc), with pagination
    /// Set 'sort' to Some(column-name, direction), e.g., Some("updated_at", Desc)
    pub async fn get_list_entries_sorted<A: Into<AllId>>(
        &self,
        list_allid: A,
        sort: Option<(&str, SortDirection)>,
        limit: usize, // number to return per call (or 0 for no limit)
        skip: usize,  // number to skip
    ) -> Result<Vec<Entry>, Error> {
        let order_by = if let Some((sort_field, sort_dir)) = sort {
            vec![OrderBy {
                column: Some(String::from(sort_field)),
                direction: sort_dir,
            }]
        } else {
            Vec::new()
        };
        let q = GetEntriesRequest {
            limit,
            skip,
            order_by,
            ..Default::default()
        };
        Ok(self.get_list_entries(list_allid, &q).await?)
    }

    /// Returns items from list - with filter and optional group-by
    /// Compared to get_list_entries, this fn allows optional grouping, and optionally can return deprecated
    /// items, and doesn't allow sorting.
    pub async fn get_list_entries_for_view(
        &self,
        list_id: ID,
        params: &GetEntriesViewRequest,
    ) -> Result<GetEntriesViewResponse, Error> {
        let url = format!("{}/lists/{}/entries/filter/list", self.url_prefix, list_id);
        let resp = self.client.post(&url).json(params).send().await?;
        self.json(resp).await
    }

    /// Update checklists
    pub async fn update_checklists<L: Into<AllId>, E: Into<AllId>>(
        &self,
        list_allid: L,
        entry_allid: E,
        checklists: Vec<Checklist>,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/lists/{}/entries/{}/checklists",
            self.url_prefix,
            list_allid.into(),
            entry_allid.into()
        );
        let data = UpdateChecklistParam { checklists };
        let resp = self.client.put(&url).json(&data).send().await?;
        self.json(resp).await
    }

    /// Delete a list entry
    pub async fn delete_entry<L: Into<AllId>, E: Into<AllId>>(
        &self,
        list_allid: L,
        entry_allid: E,
    ) -> Result<DeleteListEntryResponse, Error> {
        let url = format!(
            "{}/lists/{}/deprecated-entries/{}",
            self.url_prefix,
            list_allid.into(),
            entry_allid.into()
        );
        let resp = self.client.delete(&url).send().await?;
        self.json(resp).await
    }

    // Returns true if workspaces have been loaded
    fn have_workspaces(&self) -> Result<bool, Error> {
        let ws_cache = self.workspaces.read()?;
        Ok(!ws_cache.is_empty())
    }

    fn get_cached_list(&self, list_allid: &str) -> Result<Arc<ListInfo>, Error> {
        let list_cache = self.lists.read()?;
        let li = match list_cache.iter().find(|li| li.has_id(list_allid)) {
            Some(li) => li.clone(),
            None => return Err(Error::Other(format!("Invalid list '{}'", list_allid))),
        };
        Ok(li)
    }

    fn get_cached_workspace_allid(&self, ws_id: &str) -> Result<Arc<WorkspaceData>, Error> {
        // first check previously loaded
        let ws_cache = self.workspaces.read()?;
        let wd = match ws_cache.iter().find(|wd| wd.workspace.has_id(ws_id)) {
            Some(w) => w.clone(),
            None => return Err(Error::Other(format!("Invalid workspace_id '{}'", ws_id))),
        };
        Ok(wd)
    }

    // Returns workspace from cache, or error if there was no match for id
    // Expects that get_all_workspaces_and_lists has been called previously
    fn get_cached_workspace(&self, ws_id: ID) -> Result<Arc<WorkspaceData>, Error> {
        let ws_cache_read = self.workspaces.read()?;
        let wd = match ws_cache_read.iter().find(|w| w.workspace.id == ws_id) {
            Some(w) => w.clone(),
            None => return Err(Error::Other(format!("Invalid workspace_id '{}'", ws_id))),
        };
        Ok(wd)
    }

    /// Loads all workspaces and lists that the current user can access.
    ///
    /// Performance notes:
    /// - Repeaded calls return cached data.
    pub async fn get_all_workspaces_and_lists(&self) -> Result<Vec<Arc<Workspace>>, Error> {
        if !self.have_workspaces()? {
            let resp = self
                .client
                .get(&format!("{}/users/me/workspacesWithLists", self.url_prefix))
                .send()
                .await?;
            let ws_list: Vec<Workspace> = self.json(resp).await?;
            let mut ws_cache_write = self.workspaces.write()?;
            ws_cache_write.append(
                &mut ws_list
                    .into_iter()
                    .map(|w| Arc::new(WorkspaceData::new(w)))
                    .collect(),
            );
            // drop write lock
        }
        let ws_cache = self.workspaces.read()?;
        Ok(ws_cache.iter().map(|wd| wd.workspace.clone()).collect())
    }

    /// Returns the workspace. `ws_id` may be ID, UUID, or title.
    ///
    /// Performance notes:
    /// - If ID or UUID are known, those are preferred over title due to a potential
    ///   performance benefit.
    /// - If you expect to call this for more than one workspace during a single app session,
    ///   it is almost always more efficient to call get_all_workspaces_and_lists first,
    ///   so that all workspaces are fetched with a single hit to Zenkit servers.
    /// - If called more than once for the same workspace id (or an alternate
    ///   identifier of a previously-loaded workspace), a cached value is returned.
    /// - If get_all_workspaces_and_lists has been called previously, this function
    ///   returns cached data and does not incur additional Zenkit server hits if
    ///   ws_id is valid.
    /// - Non-matching parameters are more expensive, performance-wise. If the name
    ///   does not match cached workspaces, an additional query will always be performed.
    pub async fn get_workspace(&self, ws_id: &str) -> Result<Arc<Workspace>, Error> {
        // first check previously loaded
        if let Ok(wd) = self.get_cached_workspace_allid(ws_id) {
            return Ok(wd.workspace.clone());
        }

        // if ws_id is an int or uuid, we can get just one using the call below.
        // If it's a workspace title, we need to get all since api doesn't support get-by-name.
        if ws_id.parse::<i64>().is_ok() || crate::util::is_uuid(ws_id) {
            let url = format!("{}/workspaces/{}", self.url_prefix, ws_id);
            let resp = self.client.get(&url).send().await?;
            let ws_data = WorkspaceData::new(self.json(resp).await?);
            let mut cache_write = self.workspaces.write()?;
            let ws_copy = ws_data.workspace.clone();
            cache_write.push(Arc::new(ws_data));
            return Ok(ws_copy);
        }

        // load & cache all workspaces
        for w in self.get_all_workspaces_and_lists().await?.iter() {
            if w.has_id(ws_id) {
                return Ok(w.clone());
            }
        }
        Err(Error::Other(format!("Workspace '{}' not found", ws_id)))
    }

    /// Retrieves a list, with field definitions.
    /// list_name parameter can be string name, id, or uuid
    pub async fn get_list_info(
        &self,
        workspace_id: ID,
        list_allid: &'_ str,
    ) -> Result<Arc<ListInfo>, Error> {
        // first try list cache
        if let Ok(li) = self.get_cached_list(list_allid) {
            return Ok(li);
        }

        // list_info not cached
        // first get its containing workspace, then load fields
        let wd = match self.get_cached_workspace(workspace_id) {
            Ok(wd) => wd.workspace.clone(),
            Err(_) => {
                // wasn't cached, try to load it, or fail if invalid id
                self.get_workspace(&workspace_id.to_string()).await?
            }
        };

        let list = match wd.lists.iter().find(|l| l.has_id(list_allid)) {
            Some(list) => list.clone(),
            None => {
                return Err(Error::Other(format!(
                    "get_list_info: invalid list '{}' in workspace '{}' ({})",
                    list_allid, wd.name, workspace_id
                )))
            }
        };
        // load fields
        let fields = crate::get_api()?.get_list_elements(list.id).await?;

        let info = Arc::new(ListInfo::new(list, fields));
        let mut list_cache_write = self.lists.write()?;
        list_cache_write.push(info.clone());
        Ok(info)
    }

    /// Clears workspace cache
    pub fn clear_workspace_cache(&self) -> Result<(), Error> {
        let mut ws_cache_write = self.workspaces.write()?;
        ws_cache_write.clear();
        Ok(())
    }

    /// Clears ListInfo cache. Note: ListInfo cache contains field definitions, not items.
    pub fn clear_list_cache(&self) -> Result<(), Error> {
        let mut list_cache_write = self.lists.write()?;
        list_cache_write.clear();
        Ok(())
    }

    /// Creates a new list entry
    pub async fn create_entry(&self, list_id: ID, val: Value) -> Result<Entry, Error> {
        let url = format!("{}/lists/{}/entries", self.url_prefix, list_id);
        let resp = self.client.post(&url).json(&val).send().await?;
        self.json(resp).await
    }

    /// Creates a new webhook
    pub async fn create_webhook(&self, webhook: &NewWebhook) -> Result<Webhook, Error> {
        let url = format!("{}/webhooks", self.url_prefix);
        let resp = self.client.post(&url).json(&webhook).send().await?;
        self.json(resp).await
    }

    /// Deletes webhook
    pub async fn delete_webhook(&self, webhook_id: ID) -> Result<Webhook, Error> {
        let url = format!("{}/webhooks/{}", self.url_prefix, webhook_id);
        let resp = self.client.delete(&url).send().await?;
        self.json(resp).await
    }

    /// List webhooks created by the current user
    pub async fn get_webhooks(&self) -> Result<Vec<Webhook>, Error> {
        // found this undocumented api by trial-and-error.
        // .. tried /webooks and /workspaces/ID/webhooks before finding /users/me/webhooks
        let url = format!("{}/users/me/webhooks", self.url_prefix);
        let resp = self.client.get(&url).send().await?;
        self.json(resp).await
    }

    /// Updates list field-value
    pub async fn update_entry(
        &self,
        list_id: ID,
        entry_id: ID,
        val: Value,
    ) -> Result<Entry, Error> {
        let url = format!("{}/lists/{}/entries/{}", self.url_prefix, list_id, entry_id);
        let resp = self.client.put(&url).json(&val).send().await?;
        self.json(resp).await
    }

    /// Creates a new list Comment
    pub async fn create_list_comment(
        &self,
        list_id: ID,
        comment: &NewComment,
    ) -> Result<Activity, Error> {
        let url = format!("{}/users/me/lists/{}/activities", self.url_prefix, list_id);
        let resp = self.client.post(&url).json(&comment).send().await?;
        self.json(resp).await
    }

    /// Creates a new list entry Comment
    pub async fn create_entry_comment(
        &self,
        list_id: ID,
        entry_id: ID,
        comment: &NewComment,
    ) -> Result<Activity, Error> {
        let url = format!(
            "{}/users/me/lists/{}/entries/{}/activities",
            self.url_prefix, list_id, entry_id
        );
        let resp = self.client.post(&url).json(&comment).send().await?;
        self.json(resp).await
    }
}

// used internally for updateChecklists api
#[derive(Serialize, Debug)]
struct UpdateChecklistParam {
    checklists: Vec<Checklist>,
}

#[derive(Debug)]
struct WorkspaceData {
    workspace: Arc<Workspace>,
    user_cache: RwLock<UserCache>,
}

impl WorkspaceData {
    fn new(w: Workspace) -> Self {
        Self {
            workspace: Arc::new(w),
            user_cache: Default::default(),
        }
    }

    /// Returns user cache; loads users if cache has not been initialized,
    /// or if force_reload is true
    pub async fn ensure_user_cache(&self, force_reload: bool) -> Result<(), Error> {
        let mut write = self.user_cache.write()?;
        if write.is_empty() || force_reload {
            write.replace_all(
                crate::get_api()?
                    .get_users_raw(self.workspace.id)
                    .await?
                    .into_iter()
                    .map(Arc::new)
                    .collect(),
            );
        }
        Ok(())
    }

    /// Returns list of users in workspace
    pub async fn users(&self) -> Result<Vec<Arc<User>>, Error> {
        self.ensure_user_cache(false).await?;
        let read = self.user_cache.read()?;
        Ok(read.users())
    }

    /// Find first user matching predicate
    pub async fn find_user<P>(&self, predicate: P) -> Result<Option<Arc<User>>, Error>
    where
        P: Fn(&Arc<User>) -> bool,
    {
        self.ensure_user_cache(false).await?;
        let read = self.user_cache.read()?;
        Ok(read.find_user(predicate))
    }
}
