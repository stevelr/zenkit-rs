#![allow(missing_docs)]
/// This module defines the data types used in the ZenKit Api.
/// The names and fields are based on the definitions in the online documentation
///      (as of Sept 2020)
/// Some higher-level abstractions and apis are in the workspace module.
///
/// Some inconsistencies in the original rest api:
/// - In the ZenKit api, there are multiple conflicting standards for field and enum names.
///   - enum: sometimes initial caps, sometimes not
///   - field: sometimes camelCase, snake_case,
///     and often mixed inside same struct (comment_count, updated_by, sortOrder, ...)
/// - Sometimes joined words don't use camelCase: ('elementcategory','User:displayname')
///     (and yet other places there is 'displayName')
/// - Enum values: AccessType values are capitalized, but LoginProvider values are uncapitalized
///     even though most LoginProviders are company names
/// - Sometimes foreign id fields are type ID, sometimes type int
///     Sometimes uuid fields are type UUID, sometimes type string
///     Sometimes date fields are DateString, sometimes String
/// - Color names: color_hex (in Background) but icon_color (no _hex suffix),
///   even though value is hex. In this api, all color field names omit _hex suffix.
///
/// Changes made for consistency:
/// All struct, enum and field names are per the rust capitalization convention,
///   (Pascal case for struct/enum names, snake_case for field names)
///   Serde rules are used to map to/from the json format on a per-field/per-struct basis
/// All color fields use 'color' (no _hex suffix)
/// Whenever I felt mostly confident that a type could be made more specific, I did so
///   (String -> UUID, int -> ID, String -> DateString), etc.
///
use crate::{f32_or_str, Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{clone::Clone, default::Default, fmt, iter::Iterator};

/// Zenkit-assigned Object ID (positive int)
pub type ID = u64;
/// Zenkit-assigned short object id (string)
pub type ShortId = String; // 7-14 chars, includes at least 1 letter, may include '-'
/// Zenkit-assigned object UUID
pub type UUID = String; // RFC 4122
/// Date
pub type DateString = String; // ISO8601
/// Error code returned from zenkit api calls. See https://base.zenkit.com/docs/api/type/errorcode
pub type ErrorCode = String;

//pub type ListSettings = Value;
//pub type Notification = Value;
/// Alias for json
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

impl Into<String> for AllId {
    fn into(self) -> String {
        self.to_string()
    }
}

/// Sort direction
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

/// Element (field) data type
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
    /// Date fieid
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
    /// person or list of personss
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PredefinedCategory {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    pub name: String,
    #[serde(rename = "colorHex")]
    pub color: String,
    pub created_at: DateString,
    pub updated_at: DateString,
    pub deprecated_at: Option<DateString>,
    #[serde(rename = "elementId")]
    pub element_id: ID,
    #[serde(rename = "listId")]
    pub list_id: ID,
    //origin_data: Option<Value>, // null
    //origin_provider: Option<Value>,
    //origin_created_at
    //origin_deprecated_at
    //origin_updated_at
    #[serde(rename = "resourceTags")]
    pub resource_tags: Vec<Value>,
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ChildList {
    Child(List),
    NoList(NoAccessList),
}

#[allow(missing_docs)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NoAccessList {
    #[serde(rename = "ACCESS_DENIED")]
    pub access_denied: bool,
    pub deprecated_at: Option<String>,
    pub name: String,
    pub uuid: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ElementData {
    /// for elements of type Category, predefined_categories defines the choices
    #[serde(rename = "predefinedCategories")]
    pub predefined_categories: Option<Vec<PredefinedCategory>>,

    #[serde(default = "default_bool")]
    pub multiple: bool,

    #[serde(rename = "childList")]
    pub child_list: Option<ChildList>,
    //#[serde(rename = "childListElements")]
    //child_list_elements: Option<Vec<Element>>,
    #[serde(rename = "childListUUID")]
    pub child_list_uuid: Option<UUID>,
    #[serde(rename = "mirrorElementUUID")]
    pub mirror_element_uuid: Option<UUID>,

    /// everything else
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BusinessData {
    #[serde(flatten)]
    pub fields: JsonMap,
}

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
    /// whether to include depcecated entries
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
    pub filter: Value,
    pub group_by_element_id: ID,
    pub limit: u64,
    pub skip: u64,
    pub allow_deprecated: bool,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterCountData {
    pub total: u64,
    pub filtered_total: u64,
    /// any other fields
    #[serde(flatten)]
    pub fields: JsonMap,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntriesViewResponse {
    pub count_data: FilterCountData,
    pub count_data_per_group: Vec<FilterCountData>,
    pub list_entries: Vec<Entry>,
}

#[derive(Deserialize, Debug)]
// in api docs this is 'StrucdError', but I kept misspelling it so I'm calling it ErrorInfo
pub struct ErrorInfo {
    pub name: String,
    pub code: ErrorCode,
    /// HTTP status code
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    pub message: String,
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResult {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Access {
    pub id: Option<ID>,
    #[serde(rename = "shortId")]
    pub short_id: Option<ShortId>,
    pub uuid: Option<UUID>,
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
    #[serde(rename = "roleId")]
    pub role_id: RoleID,
    pub created_at: Option<DateString>,
    pub updated_at: Option<DateString>,
    pub deprecated_at: Option<DateString>,
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
    pub created_at: DateString,
}

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
    pub created_at: String,
    /// The timestamp at which this element was last updated
    pub updated_at: String,
    /// The timestamp at which this element was deprecated. Is null if not deprecated
    pub deprecated_at: Option<String>,
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

pub enum NumericType {
    Integer,
    Decimal,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityFilter {
    All = 0,
    SystemMessages = 1, // unused
    Comments = 2,
    Deleted = 3,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityType {
    Comment = 0,
    ResourceCreated = 1,
    ResourceUpdated = 2,
    ResourceDeprecated = 3,
    ResourceImported = 4,
    ResourceCopied = 5,
    ResourceRestored = 6,
    BulkOperationInList = 7,
    ResourceDeleted = 8,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityCreatedIn {
    Workspace = 0,
    List = 1,
    ListEntry = 2,
    ListElement = 3,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ActivityBulkAction {
    Add = 0,
    Set = 1,
    Remove = 2,
    Replace = 3,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityChangedData {
    pub bulk_action: ActivityBulkAction,
}

#[derive(Deserialize, Debug)]
pub struct Activity {
    pub id: ID,
    //#[serde(rename = "shortId")]
    //pub short_id: ShortId,
    pub uuid: UUID,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub created_in: ActivityCreatedIn,
    /// message: Only used if activity type is Comment
    pub message: Option<String>,
    #[serde(rename = "isBulk")]
    pub is_bulk: bool,
    pub bulk_rowcount: Option<u64>,
    #[serde(rename = "isShownInList", default = "default_bool")]
    pub is_shown_in_list: bool,
    #[serde(rename = "isShownInListEntry", default = "default_bool")]
    pub is_shown_in_list_entry: bool,
    #[serde(rename = "isSystemMessage", default = "default_bool")]
    pub is_system_message: bool,
    #[serde(rename = "originData")]
    pub origin_data: Option<Value>, // Value::Object

    #[serde(rename = "changedData")]
    pub changed_data: ElementChange,

    #[serde(rename = "changedDataElementId")]
    pub changed_data_element_id: Option<ID>,

    #[serde(rename = "workspaceId")]
    pub workspace_id: ID,
    //#[serde(rename = "workspaceShortId")]
    //pub workspace_short_id: ShortId,
    #[serde(rename = "workspaceUUID")]
    pub workspace_uuid: UUID,
    #[serde(rename = "workspaceName")]
    pub workspace_name: String,
    //#[serde(rename = "workspaceDescription")]
    //pub workspace_description: String,
    #[serde(rename = "workspaceDeprecated_at")]
    pub workspace_deprecated_at: Option<DateString>,

    #[serde(rename = "parentUUID")]
    pub parent_uuid: Option<UUID>,

    #[serde(rename = "listId")]
    pub list_id: ID,
    //#[serde(rename = "listShortId")]
    //pub list_short_id: ShortId,
    #[serde(rename = "listUUID")]
    pub list_uuid: UUID,
    #[serde(rename = "listName")]
    pub list_name: String,
    //#[serde(rename = "listDescription")]
    //pub list_description: String,
    #[serde(rename = "listDeprecated_at")]
    pub list_deprecated_at: Option<DateString>,

    #[serde(rename = "listEntryId")]
    pub list_entry_id: ID,
    //#[serde(rename = "listEntryShortId")]
    //pub list_entry_short_id: ShortId,
    //#[serde(rename = "listEntryName")]
    //pub list_entry_name: String,
    //#[serde(rename = "listEntryDescription")]
    //pub list_entry_description: String,
    #[serde(rename = "listEntryDeprecated_at")]
    pub list_entry_deprecated_at: Option<DateString>,

    /// Name of field updated.
    /// For convenience, coercing null/None to empty string (can be null, for example, for comments)
    #[serde(rename = "elementName", default = "empty_string")]
    pub element_name: String,

    //#[serde(rename = "elementData")]
    //pub element_data: Element,
    pub created_at: DateString,
    pub updated_at: DateString,
    pub deprecated_at: Option<DateString>,

    #[serde(rename = "userId")]
    pub user_id: ID,
    #[serde(rename = "userDisplayname")] // docs incorrectly calls it 'userDisplayName'
    pub user_display_name: String,
    #[serde(rename = "userFullname")]
    pub user_full_name: String,
    #[serde(rename = "userUsername")]
    pub user_username: String,
    #[serde(rename = "userInitials")]
    pub user_initials: String,
    #[serde(rename = "userIsImagePreferred")]
    pub user_is_image_preferred: bool,
    //#[serde(rename = "userImageLink")]
    //pub user_image_link: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangedValue<T> {
    pub value_from: Option<T>,
    pub value_to: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangedArray<T> {
    pub value_from: Vec<T>,
    pub value_to: Vec<T>,
    pub value_from_as_strings: Vec<String>,
    pub value_to_as_strings: Vec<String>,
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
    Text(ChangedValue<String>),
    Number(ChangedValue<f64>),
    Date(ChangedValue<DateValue>),
    Categories(ChangedArray<ID>),
    Persons(ChangedArray<ID>),
    References(ChangedArray<UUID>),
    Other(Value),
}

fn opt_to_string<T: ToString>(v: &Option<T>) -> String {
    match v {
        Some(x) => x.to_string(),
        None => "".to_string(),
    }
}

impl ChangedData {
    /// helper to get printable from-value
    pub fn from_as_string(&self) -> String {
        use crate::join;
        match self {
            ChangedData::Text(sval) => opt_to_string(&sval.value_from),
            ChangedData::Number(nval) => opt_to_string(&nval.value_from),
            ChangedData::Date(dval) => match dval.value_from {
                Some(DateValue {
                    date: Some(ref d),
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => d.clone(),
                Some(DateValue {
                    date: None,
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => "".to_string(),
                None => "".to_string(),
            },
            ChangedData::Categories(arr) => join(",", &arr.value_from_as_strings),
            ChangedData::Persons(arr) => join(",", &arr.value_from_as_strings),
            ChangedData::References(arr) => join(",", &arr.value_from),
            ChangedData::Other(_) => "<value>".to_string(),
        }
    }

    /// helper to get printable to-value
    pub fn to_as_string(&self) -> String {
        use crate::join;
        match self {
            ChangedData::Text(sval) => opt_to_string(&sval.value_to),
            ChangedData::Number(nval) => opt_to_string(&nval.value_to),
            ChangedData::Date(dval) => match dval.value_to {
                Some(DateValue {
                    date: Some(ref d),
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => d.clone(),
                Some(DateValue {
                    date: None,
                    end_date: _,
                    has_time: _,
                    duration: _,
                }) => "".to_string(),
                None => "".to_string(),
            },

            ChangedData::Categories(arr) => join(",", &arr.value_to_as_strings),
            ChangedData::Persons(arr) => join(",", &arr.value_to_as_strings),
            ChangedData::References(arr) => join(",", &arr.value_to),
            ChangedData::Other(_) => "<value>".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ElementChange {
    pub category_id: Option<ElementCategoryId>,
    pub data: ChangedData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewActivityElement {
    pub name: String,
    #[serde(rename = "elementCategory")]
    pub element_category: ElementCategoryId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewComment {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ChecklistItem {
    pub checked: bool,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Checklist {
    pub uuid: Option<UUID>,
    pub name: String,
    /// The content
    pub items: Vec<ChecklistItem>,
    /// if true, checked item should be hidden in display
    pub should_checked_items_be_hidden: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    pub email: String,
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deprecated_at: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
}

/// List item
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    #[serde(rename = "listId")]
    pub list_id: ID,
    pub created_at: String,
    pub updated_at: String,
    pub deprecated_at: Option<String>,
    pub created_by_displayname: Option<String>,
    pub updated_by_displayname: Option<String>,
    pub deprecated_by_displayname: Option<String>,
    pub created_by: ID,
    pub updated_by: ID,
    pub deprecated_by: Option<ID>,
    // I encountered a null displayString after creating an Entry via api without specifying it
    // To simplify coding elsewhere, coerce (rare/unlikely) null to empty string
    #[serde(rename = "displayString", default = "empty_string")]
    pub display_string: String,
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32, // sometimes negative
    pub comment_count: u64,
    pub checklists: Vec<Checklist>,
    /// User-defined fields (name_uuid, etc.) will go here
    #[serde(flatten)]
    pub fields: JsonMap,
}

impl Entry {
    /// Returns value of text string or None if undefined
    pub fn get_text_value(&self, field_uuid: &str) -> Result<Option<&str>, Error> {
        let fname = format!("{}_text", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_str())
            .unwrap_or_default())
    }

    /// Returns int value of numeric field
    pub fn get_int_value(&self, field_uuid: &str) -> Result<Option<i64>, Error> {
        let fname = format!("{}_number", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|n| n.as_i64())
            .unwrap_or_default())
    }

    /// Returns float value of numeric field
    pub fn get_float_value(&self, field_uuid: &str) -> Result<Option<f64>, Error> {
        let fname = format!("{}_number", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|n| n.as_f64())
            .unwrap_or_default())
    }

    /// Returns value of date string or None if undefined
    pub fn get_date_value(&self, field_uuid: &str) -> Result<Option<&str>, Error> {
        let fname = format!("{}_date", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_str())
            .unwrap_or_default())
    }

    /// Returns category value(s) (as strings)
    pub fn get_category_names(&self, field_uuid: &str) -> Result<Vec<&str>, Error> {
        let fname = format!("{}_categories_sort", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get("name"))
                    .filter_map(|val| val.as_str())
                    .collect()
            })
            .unwrap_or_else(Vec::new))
    }

    /// Returns category ids
    pub fn get_category_ids(&self, field_uuid: &str) -> Result<Vec<ID>, Error> {
        let fname = format!("{}_categories_sort", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get("id"))
                    .filter_map(|val| val.as_u64())
                    .collect()
            })
            .unwrap_or_else(Vec::new))
    }

    /// Returns display names of people or empty array if no people (or invalid field)
    pub fn get_person_names(&self, field_uuid: &str) -> Result<Vec<&str>, Error> {
        let fname = format!("{}_persons_sort", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get("displayname"))
                    .filter_map(|val| val.as_str())
                    .collect()
            })
            .unwrap_or_else(Vec::new))
    }

    /// Returns IDs of people or empty array if no people (or invalid field)
    pub fn get_person_ids(&self, field_uuid: &str) -> Result<Vec<ID>, Error> {
        let fname = format!("{}_persons_sort", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get("id"))
                    .filter_map(|val| val.as_u64())
                    .collect()
            })
            .unwrap_or_else(Vec::new))
    }

    /// Returns UUIDs of referent items, or empty array
    pub fn get_references(&self, field_uuid: &str) -> Result<Vec<&str>, Error> {
        let fname = format!("{}_references_sort", field_uuid);
        Ok(self
            .fields
            .get(&fname)
            .map(|v| v.as_array())
            .unwrap_or_default()
            .map(|v| {
                v.iter()
                    .filter_map(|val| val.as_object())
                    .filter_map(|val| val.get("uuid"))
                    .filter_map(|val| val.as_str())
                    .collect()
            })
            .unwrap_or_else(Vec::new))
    }
}

/// deleted item reference
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteListEntryDetail {
    pub id: ID,
    pub uuid: UUID,
    pub short_id: ShortId,
}

/// Response from delete entry
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteListEntryResponse {
    pub action: String,
    pub list_entry: DeleteListEntryDetail,
}

/// File attachment
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: Option<i64>,
    #[serde(rename = "mimetype")]
    pub mime_type: Option<String>,
    #[serde(rename = "isImage")]
    pub is_image: Option<bool>, //can be null
    #[serde(rename = "s3key")]
    pub s3_key: Option<String>,
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
    #[serde(rename = "cropParams")]
    pub crop_params: Value,
    // I uploaded an image and both height and width were null
    // their size was in metadata.height, metadata.width
    //pub width: Option<String>,
    //pub height: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deprecated_at: Option<String>,
    #[serde(rename = "uploaderId")]
    pub uploader_id: ID,
    #[serde(rename = "listId")]
    pub list_id: ID,
    #[serde(rename = "elementId")]
    pub element_id: ID,
    #[serde(rename = "cachedQuerys")]
    pub cached_queries: Value, // note spelling change
    #[serde(rename = "importError")]
    pub import_error: Option<String>,
    // undocumented field "provider"
    // None for uploads; "Link" for url type, ...
    pub provider: Option<String>,
    /// undocumented field "metadata"
    /// for jpeg: {format: "jpeg", height: number, width: number}
    pub metadata: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FilterTermModus {
    IsEmpty,
    IsNotEmpty,
    Contains,
    NotContains,
    Equals,
    NotEquals,
    StartsWith,
    NotStartsWith,
    EndsWith,
    NotEndsWith,
    InRange,
    NotInRange,
    GreaterOrEqual,
    LessOrEqual,
}

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

/// List (aka Collection)
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct List {
    pub id: ID, // List ID
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    /// Title
    #[serde(default)]
    pub name: String,
    #[serde(rename = "itemName")]
    pub item_name: Option<String>,
    #[serde(rename = "itemNamePlural")]
    pub item_name_plural: Option<String>,
    #[serde(rename = "isBuilding")]
    pub is_building: bool,
    #[serde(rename = "isMigrating")]
    pub is_migrating: bool,
    //#[serde(rename = "isPublic")]
    //pub is_public: bool, // undocumented
    #[serde(rename = "sortOrder", deserialize_with = "f32_or_str")]
    pub sort_order: f32,
    pub description: String,
    #[serde(rename = "formulaTSortOrder")]
    pub formula_tsort_order: Option<String>,
    #[serde(rename = "listFilePolicy")]
    pub list_file_policy: Option<String>,
    #[serde(rename = "originProvider")]
    pub origin_provider: Option<String>,
    #[serde(rename = "originData")]
    pub origin_data: Option<Value>,
    #[serde(rename = "defaultViewModus")]
    pub default_view_modus: i64,
    pub created_at: DateString,
    pub updated_at: DateString,
    pub deprecated_at: Option<DateString>,
    pub origin_created_at: Option<DateString>,
    pub origin_updated_at: Option<DateString>,
    pub origin_deprecated_at: Option<DateString>,
    #[serde(rename = "workspaceId")]
    pub workspace_id: ID,
    #[serde(rename = "backgroundId")]
    pub background_id: Option<String>,
    pub visibility: i64,
    #[serde(rename = "iconColor")]
    pub icon_color: Option<String>, // undocumented
    #[serde(rename = "iconBackgroundColor")]
    pub icon_background_color: Option<String>, // undocumented
    pub created_by: ID,
    //pub settings: Option<Value>, // undocumented
    //#[serde(rename = "resourceTags")]
    //pub resource_tags: Vec<ResourceTag>, // undocumented
    //     { appType: String, created_at: DateString, created_by: ID, is_owner: bool, tag: String,
    //     uuid: UUID }
    //#[serde(rename = "iconClassNames")]
    //pub icon_class_names: Option<Value>, // undocumented
}

unsafe impl Send for List {}
unsafe impl Sync for List {}

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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Workspace {
    pub id: ID,
    #[serde(rename = "shortId")]
    pub short_id: ShortId,
    pub uuid: UUID,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    /// The timestamp at which this element was created
    pub created_at: String,
    /// The timestamp at which this element was last updated
    pub updated_at: String,
    /// The timestamp at which this element was deprecated. Is null if not deprecated
    pub deprecated_at: Option<String>,
    #[serde(rename = "backgroundId")]
    pub background_id: Option<ID>,
    pub created_by: ID,

    pub lists: Vec<List>,
    // undocumented fields seen in output
    //#[serde(rename = "resourceTags")]
    //pub resource_tags: Vec<ResourceTag>,
    //pub settings: Value,
    //#[serde(rename = "app_data")]
    //pub app_data: Value,
}
unsafe impl Send for Workspace {}
unsafe impl Sync for Workspace {}

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
    pub created_at: DateString,
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

impl std::str::FromStr for TextFormat {
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
    fn from(mut v: Value) -> Result<Self, Error> {
        use ElementCategoryId::{Categories, Number, Persons, References, Text};
        if let Some(map) = v.as_object_mut() {
            if let Some(cval) = map.values_mut().take(1).next() {
                let category_id: ElementCategoryId = match cval
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
                        /*
                        return Err(Error::Other(format!(
                            "missing elementcategoryId. raw={:?}",
                            &v
                        )))
                        */
                    }
                };
                let cval = cval.take();
                let data = match category_id {
                    Text => ChangedData::Text(serde_json::from_value(cval)?),
                    Number => ChangedData::Number(serde_json::from_value(cval)?),
                    Persons => ChangedData::Persons(serde_json::from_value(cval)?),
                    References => ChangedData::References(serde_json::from_value(cval)?),
                    Categories => ChangedData::Categories(serde_json::from_value(cval)?),
                    ElementCategoryId::Date => ChangedData::Date(serde_json::from_value(cval)?),
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
                            data: ChangedData::Other(cval),
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
