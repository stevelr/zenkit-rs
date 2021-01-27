#![allow(missing_docs)]
//! This module defines the data types used in the ZenKit Api. Most are specified
//! by [Zenkit API Docs](https://base.zenkit.com/docs/api/overview/introduction),
//! and a few have been added to make the code more Rust-idiomatic.
//!
//! Structs defined here that aren't directly in Zenkit API:
//! - ListInfo - wraps a List with its field definitions, and contains business field getters and setters.
//! - Item - wraps a list Entry, and has getters and setters to simplify access to business fields.
//!   (derefs to Entry)
//! - Various structures whose names have a suffix of 'Request' or 'Response', for api parameters and
//!   responses.
//! - ChangedArray,ChangedValue - describe data changed inside an Activity object
//!
//! All struct, enum and field names follow Rust naming and capitalization convention,
//!   (Pascal case for struct/enum names, snake_case for field names)
//!   Serde rules are used to map to/from the json-defined names on a per-field/per-struct basis
//! All color fields use 'color' (no _hex suffix)
//! Whenever I felt fairly confident that a type could be made more specific, I did so
//!   (String -> UUID, int -> ID, String -> DateTime<Utc>), etc.

use crate::{f32_or_str, Error};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{clone::Clone, default::Default, fmt, iter::Iterator, str::FromStr};

// re-export
pub use crate::datetime::{DateTime, Utc};

// re-export from item and list
pub use crate::{
    item::Item,
    list::{
        fset_f, fset_i, fset_id, fset_s, fset_t, fset_vid, fset_vs, fup_f, fup_i, fup_id, fup_s,
        fup_t, fup_vid, fup_vs, FieldSetVal, FieldVal, ListInfo,
    },
};

/// Zenkit-assigned Object ID (positive int)
pub type ID = u64;
/// Zenkit-assigned short object id (string)
pub type ShortId = String; // 7-14 chars, includes at least 1 letter, may include '-'
/// Zenkit-assigned object UUID
pub type UUID = String; // RFC 4122
/// Error code returned from zenkit api calls. See https://base.zenkit.com/docs/api/type/errorcode
pub type ErrorCode = String;

//pub type ListSettings = Value;
//pub type Notification = Value;
/// string-indexed map of json values
pub type JsonMap = serde_json::map::Map<String, Value>;

/// Field is new/UI name for Element
pub type Field = Element;

fn default_bool() -> bool {
    false
}
fn empty_string() -> String {
    String::from("")
}

/// AllId is used when function parameters may accept more than one name for an object. In addition
/// to id or uuid, many functions also accept a String name (e.g., a field name)
#[derive(Debug)]
pub enum AllId {
    #[allow(non_camel_case_types)]
    /// Object ID
    ID(u64),
    /// Object short id
    ShortId(String),
    /// Object uuid
    UUID(String),
    /// Any of the above, as a string
    Any(String),
}

/// Zenkit object with ID and UUID
pub trait ZKObjectID {
    /// Returns the zenkit-assigned ID (positive int)
    fn get_id(&self) -> ID;

    /// Returns the zenkit-assigned uuid
    fn get_uuid(&self) -> &UUID;
}

impl fmt::Display for AllId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tmp: String;
        write!(
            f,
            "{}",
            match self {
                AllId::ID(val) => {
                    tmp = val.to_string();
                    &tmp
                }
                AllId::ShortId(s) => (*s).as_str(),
                AllId::UUID(s) => (*s).as_str(),
                AllId::Any(s) => (*s).as_str(),
            }
        )
    }
}

impl From<ID> for AllId {
    fn from(id: ID) -> AllId {
        AllId::ID(id)
    }
}

impl From<&ID> for AllId {
    fn from(pid: &ID) -> AllId {
        AllId::ID(*pid)
    }
}

impl From<String> for AllId {
    fn from(s: String) -> AllId {
        AllId::Any(s)
    }
}
impl From<&String> for AllId {
    fn from(s: &String) -> AllId {
        AllId::Any(s.clone())
    }
}

impl From<&'_ str> for AllId {
    fn from(s: &str) -> AllId {
        AllId::Any(s.to_string())
    }
}

//impl Into<String> for AllId {
//    fn into(self) -> String {
//       self.to_string()
//    }
//}

/// Sort direction
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

/// Element(field) data type
#[derive(
    strum_macros::Display,
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Debug,
    Copy,
    Clone,
    FromPrimitive,
)]
#[repr(u8)]
pub enum ElementCategoryId {
    /// Text field
    Text = 1,
    /// Numeric field (integer or float)
    Number = 2,
    /// URL (Link) field
    #[allow(non_camel_case_types)]
    URL = 3,
    /// Date field
    Date = 4,
    /// Checkbox (boolean) field
    Checkbox = 5,
    /// Choice/Label field
    Categories = 6,
    /// Formula
    Formula = 7,
    /// date created field
    DateCreated = 8,
    /// date updated field
    DateUpdated = 9,
    /// date deprecated field
    DateDeprecated = 10,
    /// user created by field
    UserCreatedBy = 11,
    /// user last updated field
    UserUpdatedBy = 12,
    /// user deprecated by field
    UserDeprecatedBy = 13,
    /// person or list of persons
    Persons = 14,
    /// file field (e.g., attachment)
    Files = 15,
    /// reference(s) to other items
    References = 16,
    /// Hierarchy field (sub-items)
    Hierarchy = 17,
    /// sub-entries (possibly unused?)
    SubEntries = 18,
    /// dependencies (possibly unused?)
    Dependencies = 19,
}

/// for elements of type Category, PredefinedCategory defines the choices
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PredefinedCategory {
    /// object id
    pub id: ID,
    /// object short id
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    /// object uuid
    pub uuid: UUID,
    /// category name
    pub name: String,
    /// color, in hex
    #[serde(rename = "colorHex")]
    pub color: String,
    /// date created
    pub created_at: DateTime<Utc>,
    /// date updated
    pub updated_at: DateTime<Utc>,
    /// date deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    /// element(field) id
    #[serde(rename = "elementId")]
    pub element_id: ID,
    /// containing list id
    #[serde(rename = "listId")]
    pub list_id: ID,
    //origin_data: Option<Value>, // null
    //origin_provider: Option<Value>,
    //origin_created_at
    //origin_deprecated_at
    //origin_updated_at
    /// list of resource tags
    #[serde(rename = "resourceTags")]
    pub resource_tags: Vec<Value>,
    /// sort order
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32,
}

impl ZKObjectID for PredefinedCategory {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Embedded list, if accessible
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ChildList {
    /// Embedded list
    Child(List),
    /// No accessible list
    NoList(NoAccessList),
}

/// Inaccessible lit
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NoAccessList {
    /// user does not have access to list
    #[serde(rename = "ACCESS_DENIED")]
    pub access_denied: bool,
    /// list is deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    /// list name
    pub name: String,
    /// list uuid
    pub uuid: String,
}

/// definition of type of data held by Element/field
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ElementData {
    /// for elements of type Category, predefined_categories defines the choices
    #[serde(rename = "predefinedCategories")]
    pub predefined_categories: Option<Vec<PredefinedCategory>>,

    /// true if the field can have multiple values (labels, persons, etc.)
    #[serde(default = "default_bool")]
    pub multiple: bool,

    /// embedded list, for hierarchies
    #[serde(rename = "childList")]
    pub child_list: Option<ChildList>,
    //#[serde(rename = "childListElements")]
    //child_list_elements: Option<Vec<Element>>,
    /// uuid of child list, for hierarchies
    #[serde(rename = "childListUUID")]
    pub child_list_uuid: Option<UUID>,
    /// mirror element
    #[serde(rename = "mirrorElementUUID")]
    pub mirror_element_uuid: Option<UUID>,

    /// catch-all for all other fields
    #[serde(flatten)]
    pub fields: JsonMap,
    /*
    #[serde(rename = "highlightOverdue")]
    pub highlight_overdue: Option<bool>,
    pub list_users: Option<Vec<ID>>,
    pub placeholder: Option<String>,
    #[serde(rename = "allowCreation")]
    pub allow_creation: Option<bool>,
    #[serde(rename = "allowSearch")]
    pub allow_search: Option<bool>,
    */
}

/// Definitions of business data (user-defined) fields
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BusinessData {
    /// field definitions
    #[serde(flatten)]
    pub fields: JsonMap,
}

/// Metadata about label/choice fields
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ElementDataCategories {
    #[serde(rename = "allowInlineCreation")]
    pub allow_inline_creation: bool,
    #[serde(default = "default_bool")]
    pub multiple: bool,
    #[serde(rename = "predefinedCategories")]
    pub predefined_categories: Vec<PredefinedCategory>,
}

/// Sort order for filtered query
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderBy {
    /// Column name
    pub column: Option<String>,
    //#[serde(rename = "elementId")]
    //pub element_id: Option<u64>, // element number
    /// Sort direction
    pub direction: SortDirection,
}

/// Parameters for get_list_entries
#[derive(Serialize, Deserialize, Debug)]
pub struct GetEntriesRequest {
    /// filter object to filter the response
    pub filter: Value,
    /// number of entries that will be returned
    pub limit: usize,
    /// number of entries to skip
    pub skip: usize,
    /// whether to include deprecated entries
    #[serde(rename = "allowDeprecated")]
    pub allow_deprecated: bool,
    /// sort order
    #[serde(rename = "orderBy")]
    pub order_by: Vec<OrderBy>,
}

impl Default for GetEntriesRequest {
    fn default() -> Self {
        Self {
            filter: Value::Object(JsonMap::new()),
            limit: 0,
            skip: 0,
            allow_deprecated: false,
            order_by: Vec::new(),
        }
    }
}

/// Parameters for get_entries_for_list_view
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntriesViewRequest {
    /// filter-object to filter the response
    pub filter: Value,
    /// optional group_by (for persons and categories)
    pub group_by_element_id: ID,
    /// number of items to return
    pub limit: u64,
    /// starting item number
    pub skip: u64,
    /// Allow deprecated entries to be included in the response.
    /// countData will also count deprecated entries in this case.
    /// An additional property called countDataNonDeprecated{total, filteredTotal} will be added,
    /// that does not count deprecated items.
    pub allow_deprecated: bool,
    /// Divide the entries into two groups, todo and done.
    /// This only works for lists that have the task addon activated,
    /// meaning that list.settings.tasks is set.
    /// Calling the route with this parameter set to true for a list that is not a task list
    /// will result in an error (LIST_HAS_NO_TASK_ELEMENT:C13).
    /// If everything works out, the result will contain countDataPerGroup
    /// for the keys "todo" and "done":
    /// {todo: {total: n, filteredTotal: m}, done: {total: i, filteredTotal: j}}.
    pub task_style: bool,
}

impl Default for GetEntriesViewRequest {
    fn default() -> Self {
        Self {
            filter: Value::Object(JsonMap::new()),
            group_by_element_id: 0,
            limit: 0,
            skip: 0,
            allow_deprecated: false,
            task_style: false,
        }
    }
}

/// filtered view response data
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterCountData {
    /// total number of items returned
    pub total: u64,
    /// number of filtered items
    pub filtered_total: u64,
    /// All other response fields go into the catch-all 'fields'
    #[serde(flatten)]
    pub fields: JsonMap,
}

/// Response returned from get_entries_for_list_view
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntriesViewResponse {
    /// number of filtered items returned
    pub count_data: FilterCountData,
    /// per group counts (todo, done, etc.)
    pub count_data_per_group: Vec<FilterCountData>,
    /// entries returned
    pub list_entries: Vec<Entry>,
}

/// Error details returned from Zenkit
//noinspection SpellCheckingInspection
#[derive(Deserialize, Debug)]
// in api docs this is "StrucdError", but I kept misspelling it so I'm calling it ErrorInfo
pub struct ErrorInfo {
    /// Error name
    pub name: String,
    /// Error code (see errorcode.rs)
    pub code: ErrorCode,
    /// HTTP status code
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    /// Error message
    pub message: String,
    /// Additional description
    pub description: String,
}

/// Error response returned from Zenkit
#[derive(Deserialize, Debug)]
pub struct ErrorResult {
    /// Error detailed info
    pub error: ErrorInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RoleID {
    ListOwner,
    ListAdmin,
    ListUser,
    CommentOnlyListUser,
    ReadOnlyListUser,
    WorkspaceOwner,
    WorkspaceUser,
    WorkspaceAdmin,
    CommentOnlyWorkspaceUser,
    ReadOnlyWorkspaceUser,
    OrganizationOwner,
    OrganizationUser,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LoginProvider {
    Local,
    Facebook,
    Google,
    Github,
    Slack,
    Trello,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AccessType {
    Organization,
    Workspace,
    List,
    Project,
}

/// User access type and role
#[derive(Serialize, Deserialize, Debug)]
pub struct Access {
    pub id: Option<ID>,
    #[serde(rename = "shortId")]
    pub short_id: Option<ShortId>,
    pub uuid: Option<UUID>,
    /// scope of access
    #[serde(rename = "accessType")]
    pub access_type: AccessType,
    #[serde(rename = "userId")]
    pub user_id: Option<ID>,
    #[serde(rename = "workspaceId")]
    pub workspace_id: Option<ID>,
    #[serde(rename = "listId")]
    pub list_id: Option<ID>,
    #[serde(rename = "organizationId")]
    pub organization_id: Option<ID>,
    /// Access role
    #[serde(rename = "roleId")]
    pub role_id: RoleID,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum DeviceOperatingSystem {
    Android,
    iOS,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FilterKeys {
    Text,
    NumberFrom,
    NumberTo,
    DateType,
    DateFrom,
    DateTo,
    Checked,
    FilterCategories,
    FilterPersons,
    FilterReference,
    Level,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ElementCategoryGroup {
    Control,
}

/// Metadata about field data type
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElementCategory {
    pub id: ElementCategoryId,
    pub short_id: ShortId,
    pub uuid: UUID,
    /// Human-readable name
    pub name: String,
    pub group: ElementCategoryGroup,
    /// Template for human-readable name of an element with this category
    pub display_name: String,
    /// unused
    pub placeholder_schema: String,
    /// unused
    pub container: bool,
    /// unused
    pub listable: bool,
    /// unused
    pub filterable: bool,
    pub filter_keys: Vec<String>,
    /// Entry key by which to sort values of this element category
    pub sort_key: String,
    pub searchable: bool,
    pub is_static: bool,
    pub min_width: String,
    pub width: String,
    pub max_width: String,
    pub is_width_fixed: bool,
    /// Whether elements of this category can be changed through the "Set" bulk action
    pub can_set: bool,
    /// Whether elements of this category can be changed through the "Add" bulk action
    pub can_add: bool,
    /// Whether elements of this category can be changed through the "Remove" bulk action
    pub can_remove: bool,
    /// Whether elements of this category can be changed through the "Replace" bulk action
    pub can_replace: bool,
    /// Definition of the business data of an element with this category. Only some fields are valid per category
    pub business_data_definition: Value,
    /// Defaults for an element's business data fields
    pub business_data_defaults: Value,
    /// Definition of the element data of an element with this category. Only some fields are valid per category
    pub element_data_definition: Value,
    /// Defaults for an element's element data fields
    pub element_data_defaults: Value,
    #[serde(rename = "created_at")]
    /// The timestamp at which this element category was created
    pub created_at: DateTime<Utc>,
}

/// Field definition
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Element {
    pub id: ID, // The ID
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    pub name: String,
    // documentation says displayName is unused
    //#[serde(rename = "displayName", default = "empty_string")]
    //pub display_name: String, // Unused
    /// undocumented
    pub description: Option<String>,

    /// The business data
    #[serde(rename = "businessData")]
    pub business_data: BusinessData,

    /// The element data { placeholder: "", listUsers:Option<Value> }
    #[serde(rename = "elementData")]
    pub element_data: ElementData,

    // pub blocked: Option<bool>, // Unused
    /// true if this is a list's primary element
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,

    /// true if this element was created through an automatic process, such as an import, rather than user interaction
    #[serde(rename = "isAutoCreated")]
    pub is_auto_created: bool,

    /// The sort order compared to other elements of the same list
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32,
    pub visible: bool,
    /// The timestamp at which this element was created
    pub created_at: DateTime<Utc>,
    /// The timestamp at which this element was last updated
    pub updated_at: DateTime<Utc>,
    /// The timestamp at which this element was deprecated. Is null if not deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    #[serde(rename = "elementcategory")]
    /// The element category
    pub element_category: ElementCategoryId,
    /// The ID of the list this element belongs to
    #[serde(rename = "listId")]
    pub list_id: ID,
    /// undocumented
    #[serde(rename = "visibleInPublicList")]
    pub visible_in_public_list: Option<bool>,
}

impl Element {
    /// Returns the element description, or an empty string if none was provided
    pub fn get_description(&self) -> &str {
        match &self.description {
            Some(s) => s.as_str(),
            None => "",
        }
    }

    /// lookup choice id from its name or uuid. Returns Error if there is no match
    pub fn get_choice_id(&self, choice_name: &str) -> Result<ID, Error> {
        if self.element_category == ElementCategoryId::Categories {
            if let Some(categories) = &self.element_data.predefined_categories {
                for c in categories {
                    if c.uuid == choice_name || c.name == choice_name {
                        return Ok(c.id);
                    }
                }
            }
        }
        Err(Error::Other(format!(
            "Invalid choice '{}' for field '{}'",
            choice_name, &self.name
        )))
    }

    /// Returns whether the numeric field is Integer or Decimal.
    /// Returns None if the field is not numeric
    pub fn numeric_type(&self) -> Option<NumericType> {
        if self.element_category == ElementCategoryId::Number {
            if let Some(format_val) = self.element_data.fields.get("format") {
                if let Some(format_obj) = format_val.as_object() {
                    if let Some(name_val) = format_obj.get("name") {
                        return match name_val.as_str() {
                            Some("integer") => Some(NumericType::Integer),
                            Some("decimal") => Some(NumericType::Decimal),
                            _ => None,
                        };
                    }
                }
            }
        }
        None
    }
}

impl ZKObjectID for Element {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Type of numeric field
pub enum NumericType {
    /// Integer (signed)
    Integer,
    /// Decimal - in Rust, interpreted as float
    Decimal,
}

/// Activity filter type
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityFilter {
    /// All activities (no filter)
    All = 0,
    /// System messages (unused?)
    SystemMessages = 1,
    /// Comments
    Comments = 2,
    /// Deleted
    Deleted = 3,
}

/// Type of activity
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityType {
    /// Comment
    Comment = 0,
    /// resource was created
    ResourceCreated = 1,
    /// resource was updated
    ResourceUpdated = 2,
    /// resource was deprecated
    ResourceDeprecated = 3,
    /// resource was imported
    ResourceImported = 4,
    /// resource was copied
    ResourceCopied = 5,
    /// resource was restored
    ResourceRestored = 6,
    /// bulk operation in list
    BulkOperationInList = 7,
    /// resource was deleted
    ResourceDeleted = 8,
}

/// Where activity occurred
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityCreatedIn {
    /// in workspace
    Workspace = 0,
    /// list
    List = 1,
    /// list entry (item)
    ListEntry = 2,
    /// list element (field)
    ListElement = 3,
}

/// Activity type for bulk action
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityBulkAction {
    /// add value
    Add = 0,
    /// set value
    Set = 1,
    /// remove value
    Remove = 2,
    /// replace value
    Replace = 3,
}

/// Data that changed
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityChangedData {
    ///
    pub bulk_action: ActivityBulkAction,
}

/// Activity report
//noinspection SpellCheckingInspection
#[derive(Deserialize, Debug)]
pub struct Activity {
    /// activity id
    pub id: ID,
    // short id
    //#[serde(rename = "shortId")]
    //pub short_id: ShortId,
    /// activity uuid
    pub uuid: UUID,
    /// Activity type
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// object type
    pub created_in: ActivityCreatedIn,
    /// message - only used if activity type is Comment
    pub message: Option<String>,
    /// whether activity is bulk action
    #[serde(rename = "isBulk")]
    pub is_bulk: bool,
    /// if is_bulk, this field holds number of units affected by bulk action
    pub bulk_rowcount: Option<u64>,
    //#[serde(rename = "isShownInList", default = "default_bool")]
    //pub is_shown_in_list: bool, // unused
    //#[serde(rename = "isShownInListEntry", default = "default_bool")]
    //pub is_shown_in_list_entry: bool, // unused
    //#[serde(rename = "isSystemMessage", default = "default_bool")]
    //pub is_system_message: bool, // unused
    //#[serde(rename = "originData")]
    //pub origin_data: Option<Value>// unused
    /// object describing what changed through this activity.
    #[serde(rename = "changedData")]
    pub changed_data: ElementChange,

    /// element id of changed data
    #[serde(rename = "changedDataElementId")]
    pub changed_data_element_id: Option<ID>,

    /// id of activity's subject, if it is a workspace
    #[serde(rename = "workspaceId")]
    pub workspace_id: Option<ID>,

    /// short id of activity's subject, if it is a workspace
    #[serde(rename = "workspaceShortId")]
    pub workspace_short_id: Option<ShortId>,

    /// uuid of activity's subject, if it is a workspace
    #[serde(rename = "workspaceUUID")]
    pub workspace_uuid: Option<UUID>,

    /// name of activity's subject, if it's a workspace
    #[serde(rename = "workspaceName")]
    pub workspace_name: Option<String>,

    // description of activity's subject, if it is a workspace
    //#[serde(rename = "workspaceDescription")]
    //pub workspace_description: String,
    /// date activity's subject was deprecated, if it is a workspace
    #[serde(rename = "workspaceDeprecated_at")]
    pub workspace_deprecated_at: Option<DateTime<Utc>>,

    ///
    #[serde(rename = "parentUUID")]
    pub parent_uuid: Option<UUID>,

    /// id of activity's subject, if it is a list
    #[serde(rename = "listId")]
    pub list_id: Option<ID>,

    /// short id of activity's subject, if it is a list
    #[serde(rename = "listShortId")]
    pub list_short_id: Option<ShortId>,

    /// uuid of activity's subject, if it is a list
    #[serde(rename = "listUUID")]
    pub list_uuid: Option<UUID>,
    /// name of activity's subject, if it is a list
    #[serde(rename = "listName")]
    pub list_name: Option<String>,

    //#[serde(rename = "listDescription")]
    //pub list_description: Option<String>,
    /// date list deprecated
    #[serde(rename = "listDeprecated_at")]
    pub list_deprecated_at: Option<DateTime<Utc>>,

    /// id of activity's subject, if it is a list entry
    #[serde(rename = "listEntryId")]
    pub list_entry_id: Option<ID>,

    /// uuid of activity's subject, if it is a list entry
    #[serde(rename = "listEntryUUID")]
    pub list_entry_uuid: Option<UUID>,

    //#[serde(rename = "listEntryShortId")]
    //pub list_entry_short_id: ShortId,
    /// name of activity's subject, if it is a list entry
    #[serde(rename = "listEntryName")]
    pub list_entry_name: Option<String>,
    /// description of activity's subject, if it is a list entry
    #[serde(rename = "listEntryDescription")]
    pub list_entry_description: Option<String>,
    /// date at which activity's subject was deprecated, if it is a list entry
    #[serde(rename = "listEntryDeprecated_at")]
    pub list_entry_deprecated_at: Option<DateTime<Utc>>,

    /// Name of field updated. Can be None if activity is a Comment
    #[serde(rename = "elementName")]
    pub element_name: Option<String>,

    //#[serde(rename = "elementData")]
    //pub element_data: Element,
    /// date activity created
    pub created_at: DateTime<Utc>,
    /// date activity updated
    pub updated_at: DateTime<Utc>,
    /// date activity deprecated
    pub deprecated_at: Option<DateTime<Utc>>,

    /// user id this actiity belongs to
    #[serde(rename = "userId")]
    pub user_id: ID,
    /// display name of activity's subject, if it is a user
    #[serde(rename = "userDisplayname")] // docs incorrectly calls it 'userDisplayName'
    pub user_display_name: String,
    /// full name of activity's subject, if it is a user
    #[serde(rename = "userFullname")]
    pub user_full_name: String,
    /// username of activity's subject, if it is a user
    #[serde(rename = "userUsername")]
    pub user_username: String,
    /// user initials of the activity's subject, if it is a user
    #[serde(rename = "userInitials")]
    pub user_initials: String,
    /// user image preference setting of the activity's subject, if it is a user
    #[serde(rename = "userIsImagePreferred")]
    pub user_is_image_preferred: bool,
    //#[serde(rename = "userImageLink")]
    //pub user_image_link: Option<bool>,
}

impl ZKObjectID for Activity {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Original and new value
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangedValue<T> {
    /// value before change
    pub value_from: Option<T>,
    /// value after change
    pub value_to: Option<T>,
}

/// Selection of changed data
pub enum FromTo {
    /// Value before change
    From,
    /// Value after change
    To,
}

impl<T> ChangedValue<T> {
    fn get(&self, ft: FromTo) -> &Option<T> {
        match ft {
            FromTo::From => &self.value_from,
            FromTo::To => &self.value_to,
        }
    }
}

/// Set of changed values
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangedArray<T> {
    /// Values before change
    pub value_from: Vec<T>,
    /// values after change
    pub value_to: Vec<T>,
    /// values before change, in string representation
    pub value_from_as_strings: Vec<String>,
    /// values after change, in string representation
    pub value_to_as_strings: Vec<String>,
}

impl<T> ChangedArray<T> {
    fn get(&self, ft: FromTo) -> &Vec<T> {
        match ft {
            FromTo::From => &self.value_from,
            FromTo::To => &self.value_to,
        }
    }
    fn as_strings(&self, ft: FromTo) -> &Vec<String> {
        match ft {
            FromTo::From => &self.value_from_as_strings,
            FromTo::To => &self.value_to_as_strings,
        }
    }
}

/// Value of date from or to
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DateValue {
    pub date: Option<String>,
    pub end_date: Option<String>,
    pub has_time: bool,
    pub duration: Option<String>, // ex: "4 day"
}

/// ChangedData, within an Activity, represents the previous and new data values.
/// Not all field types have been implemented as structs; the other field types
/// have change data as a (serde_json) Value.
///
/// This data will be most applicable if the activity_type is 2 (ResourceUpdated).
/// For example, if activity_type is 0 (Comment), ChangedData is Other(Null).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChangedData {
    /// change to text data
    Text(ChangedValue<String>),
    /// change to numeric data
    Number(ChangedValue<f64>),
    /// change to date field
    Date(ChangedValue<DateValue>),
    /// change to label field
    Categories(ChangedArray<ID>),
    /// change to person(s)
    Persons(ChangedArray<ID>),
    /// change to reference(s)
    References(ChangedArray<UUID>),
    /// change not covered by any of the other types
    Other(Value),
}

fn opt_to_string<T: ToString>(v: &Option<T>) -> String {
    match v {
        Some(x) => x.to_string(),
        None => "".to_string(),
    }
}

impl ChangedData {
    /// Generate printable string value for from (previous) or to (new) value
    pub fn val_to_string(&self, ft: FromTo) -> String {
        use crate::join;
        match self {
            ChangedData::Text(txt_val) => opt_to_string(txt_val.get(ft)),
            ChangedData::Number(num_val) => opt_to_string(num_val.get(ft)),
            ChangedData::Date(date_val) => match date_val.get(ft) {
                Some(DateValue {
                    date: Some(ref date),
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => date.clone(),
                Some(DateValue {
                    date: None,
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => "".to_string(),
                None => "".to_string(),
            },
            ChangedData::Categories(arr) => join(",", arr.as_strings(ft)),
            ChangedData::Persons(arr) => join(",", arr.as_strings(ft)),
            ChangedData::References(arr) => join(",", arr.get(ft)),
            ChangedData::Other(_) => "<value>".to_string(),
        }
    }
}

/// change of object field
#[derive(Debug)]
pub struct ElementChange {
    /// field type
    pub category_id: Option<ElementCategoryId>,
    /// data of changed field
    pub data: ChangedData,
}

///
#[derive(Serialize, Deserialize, Debug)]
pub struct NewActivityElement {
    /// element name
    pub name: String,
    /// field type
    #[serde(rename = "elementCategory")]
    pub element_category: ElementCategoryId,
}

/// new comment
#[derive(Serialize, Deserialize, Debug)]
pub struct NewComment {
    /// comment text
    pub message: String,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum BackgroundRole {
    UserDefault = 0,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum BackgroundType {
    AdminColor = 0,
    AdminImage = 1,
    AdminTexture = 2,
    UserImage = 3,
    WorkspaceImage = 4,
    ListImage = 5,
    AdminThemes = 6,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum BackgroundTheme {
    Dark = 0,
    Light = 1,
    Tron = 2,
    DarkTransparent = 3,
    LightTransparent = 4,
    Matrix = 5,
    IronMan = 6,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum BackgroundStyle {
    Cover = 0,
    Tile = 1,
}

/// Background theme and style
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    pub id: ID,
    pub short_id: ShortId,
    pub uuid: UUID,
    pub role: BackgroundRole,
    pub r#type: BackgroundType,
    /// id of background target. target can be a user, list, or a workspace
    pub target_id: ID,
    /// file associated with this background
    pub file_id: ShortId,
    /// background color, in hex
    #[serde(rename = "color_hex")]
    pub color: String,
    pub theme: BackgroundTheme,
    pub style: BackgroundStyle,
    pub description: String,
    /// reference link
    pub link: String,
    /// shortId of preview image file
    pub preview_file_short_id: ShortId,
}

impl ZKObjectID for Background {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// item in a checklist
#[derive(Serialize, Deserialize, Debug)]
pub struct ChecklistItem {
    /// true if item is checked
    pub checked: bool,
    /// checklist item text
    pub text: String,
}

/// Checklist field
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Checklist {
    /// checklist uuid
    pub uuid: Option<UUID>,
    /// checklist name
    pub name: String,
    /// The content
    pub items: Vec<ChecklistItem>,
    /// if true, checked item should be hidden in display
    pub should_checked_items_be_hidden: bool,
}

/// user email info
#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    /// email object id
    pub id: ID,
    /// email object short id
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    /// email object uuid
    pub uuid: UUID,
    /// email address
    pub email: String,
    /// whether this is the primary email
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    /// date created
    pub created_at: DateTime<Utc>,
    /// date updated
    pub updated_at: DateTime<Utc>,
    /// date deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    #[serde(rename = "isVerified")]
    /// whether email has been verified
    pub is_verified: bool,
}

impl ZKObjectID for Email {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// List item
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    /// object id
    pub id: ID,
    #[serde(rename = "shortId")]
    /// object short id
    pub short_id: ShortId,
    /// object uuid
    pub uuid: UUID,
    /// id of list containing this entry
    #[serde(rename = "listId")]
    pub list_id: ID,
    /// entry created date
    pub created_at: DateTime<Utc>,
    /// entry updated date
    pub updated_at: DateTime<Utc>,
    /// entry deprecated date
    pub deprecated_at: Option<DateTime<Utc>>,
    /// user that created entry
    pub created_by_displayname: Option<String>,
    /// user that updated entry
    pub updated_by_displayname: Option<String>,
    /// user that deprecated entry
    pub deprecated_by_displayname: Option<String>,
    /// id of user that created entry
    pub created_by: ID,
    /// id of user that updated entry
    pub updated_by: ID,
    /// id of user that deprecated entry
    pub deprecated_by: Option<ID>,
    /// Entry title
    // I encountered a null displayString after creating an Entry via api without specifying it
    // To simplify coding elsewhere, coerce (rare/unlikely) null to empty string
    #[serde(rename = "displayString", default = "empty_string")]
    pub display_string: String,
    /// Sort order
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32, // sometimes negative
    /// number of comments
    pub comment_count: u64,
    /// checklist items attached to this entry
    pub checklists: Vec<Checklist>,
    /// Catch-all: User-defined fields (name_uuid, etc.) will be here
    #[serde(flatten)]
    pub fields: JsonMap,
}

impl ZKObjectID for Entry {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

impl Entry {
    /// Returns value of text string or None if undefined
    pub fn get_text_value(&self, field_uuid: &str) -> Result<Option<&str>, Error> {
        let field_name = format!("{}_text", field_uuid);
        Ok(self
            .fields
            .get(&field_name)
            .map(|v| v.as_str())
            .unwrap_or_default())
    }

    /// Returns int value of numeric field
    pub fn get_int_value(&self, field_uuid: &str) -> Result<Option<i64>, Error> {
        let field_name = format!("{}_number", field_uuid);
        Ok(self
            .fields
            .get(&field_name)
            .map(|n| n.as_i64())
            .unwrap_or_default())
    }

    /// Returns float value of numeric field
    pub fn get_float_value(&self, field_uuid: &str) -> Result<Option<f64>, Error> {
        let field_name = format!("{}_number", field_uuid);
        Ok(self
            .fields
            .get(&field_name)
            .map(|n| n.as_f64())
            .unwrap_or_default())
    }

    /// Returns value of date string or None if undefined
    pub fn get_date_value(&self, field_uuid: &str) -> Result<Option<&str>, Error> {
        let field_name = format!("{}_date", field_uuid);
        Ok(self
            .fields
            .get(&field_name)
            .map(|v| v.as_str())
            .unwrap_or_default())
    }

    /// Returns label/category value(s) (as strings)
    pub fn get_category_names(&self, field_uuid: &str) -> Vec<&str> {
        self.map_values(field_uuid, "categories_sort", "name", |v| v.as_str())
    }

    /// Returns label/category ids
    pub fn get_category_ids(&self, field_uuid: &str) -> Vec<ID> {
        self.map_values(field_uuid, "categories_sort", "id", |v| v.as_u64())
    }

    //noinspection SpellCheckingInspection
    /// Returns display names of people referenced by this field
    pub fn get_person_names(&self, field_uuid: &str) -> Vec<&str> {
        self.map_values(field_uuid, "persons_sort", "displayname", |v| v.as_str())
    }

    /// Returns IDs of people referenced by this field
    pub fn get_person_ids(&self, field_uuid: &str) -> Vec<ID> {
        self.map_values(field_uuid, "persons_sort", "id", |v| v.as_u64())
    }

    /// Returns UUIDs of referent items, or empty array
    pub fn get_references(&self, field_uuid: &str) -> Vec<&str> {
        self.map_values(field_uuid, "references_sort", "uuid", |v| v.as_str())
    }

    // obtain Vec<value> from array of maps of key-value
    fn map_values<'v, T>(
        &'v self,
        field_uuid: &'_ str,
        field_kind: &'_ str,
        key: &'_ str,
        pred: fn(&'v Value) -> Option<T>,
    ) -> Vec<T> {
        let field_name = format!("{}_{}", field_uuid, field_kind);
        self.fields
            .get(&field_name)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get(key))
                    .filter_map(|val| pred(val))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }
}

/// deleted item reference
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteListEntryDetail {
    /// object id
    pub id: ID,
    /// object uuid
    pub uuid: UUID,
    /// object short id
    pub short_id: ShortId,
}

impl ZKObjectID for DeleteListEntryDetail {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Response from delete entry
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteListEntryResponse {
    ///
    pub action: String,
    ///
    pub list_entry: DeleteListEntryDetail,
}

/// File attachment
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    /// object id
    pub id: ID,
    /// object short id
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    /// object uuid
    pub uuid: UUID,
    /// file name
    #[serde(rename = "fileName")]
    pub file_name: String,
    /// file size
    pub size: Option<i64>,
    /// file mime type
    #[serde(rename = "mimetype")]
    pub mime_type: Option<String>,
    /// image flag
    #[serde(rename = "isImage")]
    pub is_image: Option<bool>, //can be null
    /// AWS S3 identity
    #[serde(rename = "s3key")]
    pub s3_key: Option<String>,
    /// file url
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
    /// crop parameters
    #[serde(rename = "cropParams")]
    pub crop_params: Value,
    // I uploaded an image and both height and width were null
    // their size was in metadata.height, metadata.width
    //pub width: Option<String>,
    //pub height: Option<String>,
    /// date created
    pub created_at: DateTime<Utc>,
    /// date updated
    pub updated_at: DateTime<Utc>,
    /// date deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    /// user that uploaded
    #[serde(rename = "uploaderId")]
    pub uploader_id: ID,
    /// containing list
    #[serde(rename = "listId")]
    pub list_id: ID,
    /// field id
    #[serde(rename = "elementId")]
    pub element_id: ID,
    /// queries
    #[serde(rename = "cachedQuerys")]
    pub cached_queries: Value, // note spelling change
    /// error during import
    #[serde(rename = "importError")]
    pub import_error: Option<String>,
    /// file provider: link for url type, None for uploads (undocumented)
    pub provider: Option<String>,
    /// undocumented field "metadata"
    /// for jpeg: {format: "jpeg", height: number, width: number}
    pub metadata: Option<Value>,
}

impl ZKObjectID for File {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Filter expression term
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FilterTermModus {
    /// empty field
    IsEmpty,
    /// non-empty field
    IsNotEmpty,
    /// field contains text
    Contains,
    /// field does not contain text
    NotContains,
    /// field equals text
    Equals,
    /// field does ot equal text
    NotEquals,
    /// field starts with text
    StartsWith,
    /// field does not start with text
    NotStartsWith,
    /// field ends with text
    EndsWith,
    /// field does not end with text
    NotEndsWith,
    /// field has value within range
    InRange,
    /// field value is not within range
    NotInRange,
    /// field is greater or equal to value
    GreaterOrEqual,
    /// field is less than or equal to value
    LessOrEqual,
}

/// date expression term
//noinspection SpellCheckingInspection
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum DateFilterTermModus {
    Any = 0,
    ThisYear = 1,
    ThisMonth = 2,
    ThisWeek = 3,
    Yesterday = 4,
    Today = 5,
    Tomorrow = 6,
    NextWeek = 7,
    NextMonth = 8,
    NextYear = 9,
    Custom = 10,
    LastWeek = 11,
    LastMonth = 12,
    LastYear = 13,
    Empty = 14,
    NotEmpty = 15,
}

/// List (aka Collection).
/// See also ListInfo, which wraps a List with field definitions,
/// to provide getters and setters for user-defined fields
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct List {
    /// object id
    pub id: ID, // List ID
    /// object short id
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    /// object uuid
    pub uuid: UUID,
    /// list name / title
    #[serde(default)]
    pub name: String,
    /// optional name for list item (defaults to list name)
    #[serde(rename = "itemName")]
    pub item_name: Option<String>,
    /// optional plural name for list item (defaults to list name)
    #[serde(rename = "itemNamePlural")]
    pub item_name_plural: Option<String>,
    ///
    #[serde(rename = "isBuilding")]
    pub is_building: bool,
    ///
    #[serde(rename = "isMigrating")]
    pub is_migrating: bool,
    //#[serde(rename = "isPublic")]
    //pub is_public: bool, // undocumented
    /// list sort order
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32,
    /// description of list
    pub description: String,
    ///
    #[serde(rename = "formulaTSortOrder")]
    pub formula_tsort_order: Option<String>,
    ///
    #[serde(rename = "listFilePolicy")]
    pub list_file_policy: Option<String>,
    ///
    #[serde(rename = "originProvider")]
    pub origin_provider: Option<String>,
    ///
    #[serde(rename = "originData")]
    pub origin_data: Option<Value>,
    ///
    #[serde(rename = "defaultViewModus")]
    pub default_view_modus: i64,
    /// date list created
    pub created_at: DateTime<Utc>,
    /// date list last updated
    pub updated_at: DateTime<Utc>,
    /// date list deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    ///
    pub origin_created_at: Option<DateTime<Utc>>,
    ///
    pub origin_updated_at: Option<DateTime<Utc>>,
    ///
    pub origin_deprecated_at: Option<DateTime<Utc>>,
    #[serde(rename = "workspaceId")]
    /// id of workspace containing list
    pub workspace_id: ID,
    #[serde(rename = "backgroundId")]
    ///
    pub background_id: Option<String>,
    ///
    pub visibility: i64,
    ///
    #[serde(rename = "iconColor")]
    pub icon_color: Option<String>, // undocumented
    ///
    #[serde(rename = "iconBackgroundColor")]
    pub icon_background_color: Option<String>, // undocumented
    /// id of user that created list
    pub created_by: ID,
    //pub settings: Option<Value>, // undocumented
    //#[serde(rename = "resourceTags")]
    //pub resource_tags: Vec<ResourceTag>, // undocumented
    //     { appType: String, created_at: DateTime<Utc>, created_by: ID, is_owner: bool, tag: String,
    //     uuid: UUID }
    //#[serde(rename = "iconClassNames")]
    //pub icon_class_names: Option<Value>, // undocumented
}

impl ZKObjectID for List {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

impl List {
    /// Returns true if the list has the id, uuid, shortId, or name of the parameter
    pub fn has_id(&self, id: &str) -> bool {
        self.uuid == id || self.name == id || self.short_id == id || self.id.to_string() == id
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<7} {} {}", self.id, self.uuid, self.name)
    }
}

/// prototype
#[derive(Serialize, Deserialize, Debug)]
pub struct ListPrototype {
    /// list name
    pub name: String,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ListVisibility {
    ListMembersOnly = 0,
    ListMembersAndWorkspaceMembers = 1,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum NotificationType {
    PersonAdded,
    ListShare,
    WorkspaceShare,
    Subscription,
    Reminder,
    Mention,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinNotification {
    pub id: ID,
    pub short_id: ShortId,
    pub uuid: UUID,
    pub list_id: ID,
    pub workspace_id: ID,
    pub list_entry_id: ID,
    pub element_id: ID,
    pub activity_id: ID,
}

/// User profile data
//noinspection SpellCheckingInspection
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    #[serde(rename = "displayname")]
    pub display_name: String,
    #[serde(rename = "fullname")]
    pub full_name: String,
    pub initials: String,
    #[serde(rename = "username")]
    pub user_name: String,
    #[serde(rename = "backgroundId")]
    pub background_id: Option<ID>,
    pub api_key: Option<String>,
    #[serde(rename = "imageLink")]
    pub image_link: Option<Value>,
    #[serde(rename = "isImagePreferred")]
    pub is_image_preferred: bool,
    pub anonymous: Option<bool>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    #[serde(rename = "isSuperAdmin")]
    pub is_super_admin: Option<bool>,
    pub registered_at: Option<Value>,
    pub trello_token: Option<String>,
    pub settings: Option<Value>,
    #[serde(rename = "emailCount")]
    pub email_count: u64,
    // pub emails: Vec<Email>,
}

impl ZKObjectID for User {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Workspace
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    /// workspace id
    pub id: ID,
    /// workspace short id
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    /// workspace uuid
    pub uuid: UUID,
    /// workspace name
    pub name: String,
    /// workspace description
    pub description: Option<String>,
    /// whether this is user's default workspace
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    /// The timestamp at which this element was created
    pub created_at: DateTime<Utc>,
    /// The timestamp at which this element was last updated
    pub updated_at: DateTime<Utc>,
    /// The timestamp at which this element was deprecated. Is null if not deprecated
    pub deprecated_at: Option<DateTime<Utc>>,
    /// workspace background theme
    #[serde(rename = "backgroundId")]
    pub background_id: Option<ID>,
    /// workspace creator user id
    pub created_by: ID,

    /// lists in workspace
    pub lists: Vec<List>,
    // undocumented fields seen in output
    //#[serde(rename = "resourceTags")]
    //pub resource_tags: Vec<ResourceTag>,
    //pub settings: Value,
    //#[serde(rename = "app_data")]
    //pub app_data: Value,
}

impl ZKObjectID for Workspace {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

impl Workspace {
    /// Returns the workspace description, or an empty string if none was provided
    pub fn get_description(&self) -> &str {
        match &self.description {
            Some(s) => s.as_str(),
            None => "",
        }
    }

    /// Returns true if the workspace has the id, uuid, shortId, or name of the parameter
    pub fn has_id(&self, id: &str) -> bool {
        self.uuid == id || self.name == id || self.short_id == id || self.id.to_string() == id
    }

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<7} {} {}", self.id, self.uuid, self.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceTag {
    pub uuid: UUID,
    pub tag: String,
    #[serde(rename = "appType")]
    pub app_type: String,
    #[serde(rename = "isOwner")]
    pub is_owner: bool,
    pub created_by: ID,
    pub created_at: DateTime<Utc>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextFormat {
    Plain,
    #[allow(non_camel_case_types)]
    HTML,
    Markdown,
}

impl fmt::Display for TextFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TextFormat::Plain => "plain",
                TextFormat::HTML => "html",
                TextFormat::Markdown => "markdown",
            }
        )
    }
}

impl Default for TextFormat {
    fn default() -> Self {
        TextFormat::Plain
    }
}

impl FromStr for TextFormat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(TextFormat::Plain),
            "markdown" => Ok(TextFormat::Markdown),
            "html" => Ok(TextFormat::HTML),
            _ => Err(Error::Other(format!(
                "TextFormat parse error: '{}' not 'plain', 'html', or 'markdown'",
                s
            ))),
        }
    }
}

/// Webhook trigger
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum WebhookTriggerType {
    Entry = 0,
    Activity = 1,
    Notification = 2,
    SystemMessage = 3,
    Comment = 4,
    Element = 5,
}

/// Webhook definition
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub id: ID,
    pub short_id: ShortId,
    pub uuid: UUID,
    pub trigger_type: WebhookTriggerType,
    pub user_id: ID,
    //pub created_by: ID, // undocumented
    pub workspace_id: Option<ID>,
    pub list_id: Option<ID>,
    pub list_entry_id: Option<ID>,
    pub url: String,
    pub provider: Option<String>,
    pub locale: String,
    pub element_id: Option<ID>,
}

impl ZKObjectID for Webhook {
    fn get_id(&self) -> ID {
        self.id
    }
    fn get_uuid(&self) -> &UUID {
        &self.uuid
    }
}

/// Parameter for creating a new webhook
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewWebhook {
    pub trigger_type: WebhookTriggerType,
    pub url: String,
    /// restrict webhook to this workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<ID>,
    /// restrict webhook to this list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<ID>,
    /// restrict webhook to this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_entry_id: Option<ID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<ID>,
    pub locale: String,
}

/// Application OAuth client configuration
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClient {
    /// name of your app
    pub client_name: String,
    /// website of app, Optional
    pub client_url: Option<String>,
    /// OAuth 2.0 Authorization code grant redirect URI
    pub redirect_uri: String,
}

/// OAuth response data
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OAuthResponse {
    pub client_id: String,
    pub client_secret: String,
}

/// Return value from get_shared_access
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SharedAccesses {
    pub list_ids: Vec<ID>,
    pub workspace_ids: Vec<ID>,
}

/// Type of data change
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
#[serde(rename = "lowercase")]
pub enum UpdateAction {
    /// Replace old values with new values
    Replace,
    /// Append new values to old values (assumes multi-valued)
    Append,
    /// Removes value(s)
    Remove,
    /// No update action
    Null,
}

impl fmt::Display for UpdateAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UpdateAction::Replace => "replace",
                UpdateAction::Append => "append",
                UpdateAction::Remove => "remove",
                _ => "",
            }
        )
    }
}

/// Converts character ('=', '+', '-') to UpdateAction (Replace, Append, Remove)
/// any other char maps to Null
impl From<char> for UpdateAction {
    fn from(c: char) -> Self {
        match c {
            '=' => UpdateAction::Replace,
            '+' => UpdateAction::Append,
            '-' => UpdateAction::Remove,
            _ => UpdateAction::Null,
        }
    }
}

impl ElementChange {
    /// Convert Json Value to ChangedData. This is called during deserialization
    /// Most of the parsing logic is devoted to handling ResourceUpdate - activity type 2.
    /// For other activity types (for example, Comment, New Item, etc.)
    /// return the data as ChangedData::Other(Value).
    //noinspection SpellCheckingInspection
    fn from(mut v: Value) -> Result<Self, Error> {
        use ElementCategoryId::{Categories, Number, Persons, References, Text};
        if let Some(map) = v.as_object_mut() {
            if let Some(change_val) = map.values_mut().take(1).next() {
                let category_id: ElementCategoryId = match change_val
                    .get("elementcategoryId")
                    .map(|n| n.as_u64())
                    .unwrap_or_default()
                    .map(ElementCategoryId::from_u64)
                    .unwrap_or_default()
                {
                    Some(cid) => cid,
                    None => {
                        return Ok(ElementChange {
                            category_id: None,
                            data: ChangedData::Other(v),
                        });
                    }
                };
                let change_val = change_val.take();
                let data = match category_id {
                    Text => ChangedData::Text(serde_json::from_value(change_val)?),
                    Number => ChangedData::Number(serde_json::from_value(change_val)?),
                    Persons => ChangedData::Persons(serde_json::from_value(change_val)?),
                    References => ChangedData::References(serde_json::from_value(change_val)?),
                    Categories => ChangedData::Categories(serde_json::from_value(change_val)?),
                    ElementCategoryId::Date => {
                        ChangedData::Date(serde_json::from_value(change_val)?)
                    }
                    // not yet implemented
                    ElementCategoryId::URL
                    | ElementCategoryId::Checkbox
                    | ElementCategoryId::Formula
                    | ElementCategoryId::DateCreated
                    | ElementCategoryId::DateUpdated
                    | ElementCategoryId::DateDeprecated
                    | ElementCategoryId::UserCreatedBy
                    | ElementCategoryId::UserUpdatedBy
                    | ElementCategoryId::UserDeprecatedBy
                    | ElementCategoryId::Files
                    | ElementCategoryId::Hierarchy
                    | ElementCategoryId::SubEntries
                    | ElementCategoryId::Dependencies => {
                        // for unimplemented categories, use None to signal value needs to be decoded
                        return Ok(ElementChange {
                            category_id: None,
                            data: ChangedData::Other(change_val),
                        });
                    }
                };
                return Ok(ElementChange {
                    category_id: Some(category_id),
                    data,
                });
            }
        }
        // null value is expected when activity type is a Comment, and no fields changed
        if v.is_null() {
            return Ok(ElementChange {
                category_id: None,
                data: ChangedData::Other(Value::Null),
            });
        }
        Err(Error::Other(format!(
            "Parse error in changed_data val={:#?}",
            v
        )))
    }
}

impl<'de> Deserialize<'de> for ElementChange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        Ok(ElementChange::from(v).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other(&e.to_string()),
                &"changed_data",
            )
        })?)
    }
}
