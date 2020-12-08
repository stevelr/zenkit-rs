//! Lists
//!

use crate::{
    types::{
        AllId, ElementCategoryId, Entry, Field, GetEntriesRequest, JsonMap, List, NumericType,
        TextFormat, UpdateAction, ID, UUID,
    },
    Error, Item, Result,
};
use serde_json::{json, Value};
use std::{fmt, iter::Iterator, rc::Rc, string::ToString};
use uuid::Uuid;

/// A read-only reference to a List and its fields
/// To modify list field definitions, use methods of workspace ..
#[derive(Debug)]
pub struct ListInfo {
    list: List,
    fields: Vec<Field>,
}

impl fmt::Display for ListInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ListInfo({},{},{}, nFields:{}, ws:{})",
            self.list.id,
            self.list.uuid,
            self.list.name,
            self.fields.len(),
            self.list.workspace_id,
        )
    }
}

impl ListInfo {
    pub(crate) fn new(list: List, fields: Vec<Field>) -> Self {
        ListInfo { list, fields }
    }

    /// Returns the inner list. You can also use the implied deref.
    pub fn list(&self) -> &List {
        &self.list
    }

    /// Returns the unique (int) ID of the list
    pub fn get_id(&self) -> ID {
        self.list.id
    }

    /// Returns the uuid of the list
    pub fn get_uuid(&self) -> &UUID {
        &self.list.uuid
    }

    /// Returns true if the list has the id, uuid, shortId, or name of the parameter
    pub fn has_id(&self, id: &str) -> bool {
        self.list.has_id(id)
    }

    /// Returns a list item by id or uuid, or None if it doesn't exist
    pub async fn get_item<A: Into<AllId>>(&'_ self, item_uid: A) -> Result<Rc<Item<'_>>, Error> {
        let item = crate::get_api()?
            .get_entry(self.get_id(), item_uid)
            .await
            .map(|entry| Item::new(entry, &self.list.name, self.list.id, &self.fields))?;
        Ok(Rc::new(item))
    }

    /// Returns field (definition) given its name, id, or uuid
    // This is a duplicate of the function in impl Item
    pub fn get_field(&self, field_id: &str) -> Result<&Field, Error> {
        self.fields
            .iter()
            .find(|e| e.name == field_id || e.uuid == field_id || e.id.to_string() == field_id)
            .ok_or_else(|| {
                Error::Other(format!(
                    "Invalid field '{}' in list {}",
                    field_id, self.list.name,
                ))
            })
    }

    /// Returns vec of fields
    pub fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    /// fetch all items of the list, unsorted
    pub async fn get_items(&'_ self) -> Result<Vec<Rc<Item<'_>>>, Error> {
        let max_items = 500usize;
        let mut start_index = 0usize;
        let mut items: Vec<Rc<Item<'_>>> = Vec::new();

        loop {
            // get the items and build the index
            let entries: Vec<Entry> = crate::get_api()?
                .get_list_entries(
                    self.get_uuid(),
                    &GetEntriesRequest {
                        limit: max_items,
                        skip: start_index,
                        ..Default::default()
                    },
                )
                .await?;
            if entries.is_empty() {
                break;
            }
            start_index += entries.len();
            let mut new_items = entries
                .into_iter()
                .map(|entry| self.new_item(entry))
                .collect();
            items.append(&mut new_items);
        }
        Ok(items)
    }

    fn new_item(&self, entry: Entry) -> Rc<Item> {
        Rc::new(Item::new(
            entry,
            &self.list.name,
            self.list.id,
            &self.fields,
        ))
    }

    /// Create a new list item.
    /// list parameter is list name, uuid, or id as string,
    /// For values, you can use the fset_* helper functions (UpdateAction not required for new
    /// items)
    /// Returned item has additional fields filled in by system (id, uuid, created_at, etc.)
    pub async fn create_item(&'_ self, values: Vec<FieldSetVal>) -> Result<Rc<Item<'_>>, Error> {
        let mut map = JsonMap::new();

        for f_set in values.into_iter() {
            self.generic_set(&mut map, f_set).await?;
        }
        let entry = crate::get_api()?
            .create_entry(self.get_id(), Value::Object(map))
            .await?;
        Ok(self.new_item(entry))
    }

    /// Updates an item with one or more field changes
    /// For values, you should use the fup_* (rather than fset_*) helper functions to ensure UpdateAction
    /// is set correctly.
    /// List and fields are 'Any' type: name, id, or uuid.
    /// For Person field, value can be name.
    /// For choice field, value can be choice (category) name.
    /// Returns updated object
    pub async fn update_item(
        &'_ self,
        item_id: ID,
        values: Vec<FieldSetVal>,
    ) -> Result<Rc<Item<'_>>, Error> {
        let mut map = JsonMap::new();
        for f_set in values.into_iter() {
            self.generic_set(&mut map, f_set).await?;
        }
        let entry = crate::get_api()?
            .update_entry(self.get_id(), item_id, Value::Object(map))
            .await?;
        Ok(self.new_item(entry))
    }

    /// Add field settings to object map.
    /// For choice field, value can be choice (category) name.
    /// For Person field, value can be name.
    /// Note: if using any form of person lookup (by name or by uuid),
    /// you must have previously made a one-time call to init_user_cache(),
    /// or this method will return Error::NotInitialized.
    async fn generic_set(&self, obj: &mut JsonMap, field_val: FieldSetVal) -> Result<(), Error> {
        use FieldVal::{ArrID, ArrStr, Float, Formatted, Int, Str};
        use UpdateAction::{Null, Replace};

        let field = self.get_field(&field_val.0)?;
        match (field.element_category, field_val.1, field_val.2) {
            (ElementCategoryId::Text, Formatted(s, fmt), Replace)
            | (ElementCategoryId::Text, Formatted(s, fmt), Null) => {
                obj.insert(format!("{}_{}", field.uuid, "text"), Value::String(s));
                obj.insert(
                    format!("{}_{}", field.uuid, "textType"),
                    Value::String(fmt.to_string()),
                );
            }
            (ElementCategoryId::Text, Str(s), Replace)
            | (ElementCategoryId::Text, Str(s), Null) => {
                obj.insert(format!("{}_{}", field.uuid, "text"), Value::String(s));
            }
            (ElementCategoryId::Number, Int(n), Replace)
            | (ElementCategoryId::Number, Int(n), Null) => {
                let num = serde_json::Number::from(n);
                obj.insert(format!("{}_{}", field.uuid, "number"), Value::Number(num));
            }
            (ElementCategoryId::Number, Float(n), Replace)
            | (ElementCategoryId::Number, Float(n), Null) => {
                let num = serde_json::Number::from_f64(n).ok_or_else(|| {
                    Error::Other("Float values cannot be Infinite or NaN".to_string())
                })?;
                obj.insert(format!("{}_{}", field.uuid, "number"), Value::Number(num));
            }
            // for convenience (esp. for cli tools), support coersion from str to number
            // We perform extra error checking here to provide specific error messages
            (ElementCategoryId::Number, Str(s), Replace)
            | (ElementCategoryId::Number, Str(s), Null) => {
                let num = match field.numeric_type() {
                    Some(NumericType::Integer) => {
                        let ival: i64 = s.parse::<i64>().map_err(|_| {
                            Error::Other(format!(
                                "Invalid int value {} for field {}",
                                s, field.name
                            ))
                        })?;
                        serde_json::Number::from(ival)
                    }
                    Some(NumericType::Decimal) => {
                        let fval: f64 = s.parse::<f64>().map_err(|_| {
                            Error::Other(format!(
                                "Invalid float value {} for field {}",
                                s, field.name
                            ))
                        })?;
                        serde_json::Number::from_f64(fval).ok_or_else(|| {
                            Error::Other("Float values cannot be Infinite or NaN".to_string())
                        })?
                    }
                    None => {
                        return Err(Error::Other(format!(
                            "Unknown numeric type at field {}",
                            field.name
                        )));
                    }
                };
                obj.insert(format!("{}_{}", field.uuid, "number"), Value::Number(num));
            }
            (ElementCategoryId::URL, Str(s), Replace) | (ElementCategoryId::URL, Str(s), Null) => {
                obj.insert(format!("{}_{}", field.uuid, "link"), Value::String(s));
            }
            (ElementCategoryId::Date, Str(s), Replace)
            | (ElementCategoryId::Date, Str(s), Null) => {
                obj.insert(format!("{}_{}", field.uuid, "date"), Value::String(s));
            }
            (ElementCategoryId::Persons, Str(s), act) => {
                let api = crate::get_api()?;
                match api.get_user_id(self.list.workspace_id, &s).await? {
                    Some(uid) => {
                        obj.insert(format!("{}_{}", field.uuid, "persons"), json!(vec![uid]));
                    }
                    None => {
                        return Err(Error::Other(format!("User not found: '{}'", s)));
                    }
                }
                if act != Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Persons, ArrStr(pvec), act) => {
                if !field.element_data.multiple && pvec.len() > 1 {
                    return Err(Error::Other(format!(
                        "Field {} can't accept more than one person but {} were provided",
                        field.name,
                        pvec.len()
                    )));
                }
                let api = crate::get_api()?;
                let mut v = Vec::<ID>::new();
                for pname in pvec.iter() {
                    v.push(
                        match api.get_user_id(self.list.workspace_id, &pname).await? {
                            Some(uid) => uid,
                            None => {
                                return Err(Error::Other(format!("User not found: '{}'", pname)));
                            }
                        },
                    );
                }
                obj.insert(format!("{}_{}", field.uuid, "persons"), json!(v));
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Persons, Int(pid), act) => {
                obj.insert(
                    format!("{}_{}", field.uuid, "persons"),
                    json!(vec![pid as u64]),
                );
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Persons, ArrID(pvec), act) => {
                if !field.element_data.multiple && pvec.len() > 1 {
                    return Err(Error::Other(format!(
                        "Field {} can't accept more than one person but {} were provided",
                        field.name,
                        pvec.len()
                    )));
                }
                obj.insert(format!("{}_{}", field.uuid, "persons"), json!(pvec));
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Categories, Str(s), act) => {
                obj.insert(
                    format!("{}_{}", field.uuid, "categories"),
                    json!(vec![field.get_choice_id(&s)?]),
                );
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Categories, Int(cid), act) => {
                obj.insert(
                    format!("{}_{}", field.uuid, "categories"),
                    json!(vec![cid as u64]),
                );
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Categories, ArrID(cvec), act) => {
                if !field.element_data.multiple && cvec.len() > 1 {
                    return Err(Error::Other(format!(
                        "Field {} can't accept more than one category but {} were provided",
                        field.name,
                        cvec.len()
                    )));
                }
                obj.insert(format!("{}_{}", field.uuid, "categories"), json!(cvec));
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::Categories, ArrStr(cvec), act) => {
                if !field.element_data.multiple && cvec.len() > 1 {
                    return Err(Error::Other(format!(
                        "Field {} can't accept more than one label but {} were provided",
                        field.name,
                        cvec.len()
                    )));
                }
                obj.insert(
                    format!("{}_{}", field.uuid, "categories"),
                    json!(cvec
                        .iter()
                        .map(|cat| field.get_choice_id(cat))
                        .collect::<Result<Vec<ID>, Error>>()?),
                );
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::References, Str(s), act) => {
                check_uuid(&s, &field.name)?;
                obj.insert(format!("{}_{}", field.uuid, "references"), json!(vec![s]));
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::References, Int(rid), act) => {
                obj.insert(
                    format!("{}_{}", field.uuid, "references"),
                    json!(vec![rid as u64]),
                );
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (ElementCategoryId::References, ArrStr(rvec), act) => {
                if !field.element_data.multiple && rvec.len() > 1 {
                    return Err(Error::Other(format!(
                        "Field {} can't accept more than one reference but {} were provided",
                        field.name,
                        rvec.len()
                    )));
                }
                for uuid in rvec.iter() {
                    check_uuid(uuid, &field.name)?;
                }
                obj.insert(format!("{}_{}", field.uuid, "references"), json!(rvec));
                if act != UpdateAction::Null {
                    obj.insert(String::from("updateAction"), Value::String(act.to_string()));
                }
            }
            (typ, value, action) => {
                return Err(Error::Other(format!(
                    "Invalid value ({:?}) or action ({:?}) for field {} (type {:?})",
                    value, action, &field.name, typ
                )));
            }
        }
        Ok(())
    }

    /// Adds comment to the item
    pub async fn add_item_comment<A: Into<AllId>>(
        &self,
        item_allid: A,
        message: String,
    ) -> Result<(), Error> {
        let item = self.get_item(item_allid).await?;
        let comment = crate::types::NewComment { message };
        let _activity = crate::get_api()?
            .create_entry_comment(self.list.id, item.as_entry().id, &comment)
            .await?;
        Ok(())
    }

    /// Adds comment to the list
    pub async fn add_list_comment(&self, message: String) -> Result<(), Error> {
        let comment = crate::types::NewComment { message };
        let _ = crate::get_api()?
            .create_list_comment(self.list.id, &comment)
            .await?;
        Ok(())
    }
}

/// Access inner list
impl std::ops::Deref for ListInfo {
    type Target = List;

    /// Access inner list
    fn deref(&self) -> &List {
        &self.list
    }
}

/// Returns true if the parameter is a valid uuid
/// It's possible that the uuid crate supports formats that aren't supported by Zenkit.
/// To reduce the chance of false positives, we also require the input to be 36 chars.
/// The fname parameter is used for error messages
pub(crate) fn check_uuid(uuid: &str, fname: &str) -> Result<(), Error> {
    match uuid.len() == 36 && Uuid::parse_str(uuid).is_ok() {
        true => Ok(()),
        false => Err(Error::Other(format!(
            "Not a valid uuid '{}' for field '{}'",
            uuid, fname
        ))),
    }
}

/// Parameters for setting a field
pub type FieldSetVal = (String, FieldVal, UpdateAction);

/// Set field with String value (Text, or number-to-string)
#[inline]
pub fn fset_s(fname: &str, val: &str) -> FieldSetVal {
    fup_s(fname, val, UpdateAction::Null)
}

/// Update field with String and action
#[inline]
pub fn fup_s(fname: &str, val: &str, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::Str(val.to_string()), act)
}

/// Set field with ID
#[inline]
pub fn fset_id(fname: &str, val: ID) -> FieldSetVal {
    fup_id(fname, val, UpdateAction::Null)
}

/// Update field with ID
#[inline]
pub fn fup_id(fname: &str, val: ID, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::Int(val as i64), act)
}

/// Set field with formatted text
#[inline]
pub fn fset_t(fname: &str, val: String, fmt: TextFormat) -> FieldSetVal {
    fup_t(fname, val, fmt, UpdateAction::Null)
}

/// Update field with formatted text
#[inline]
pub fn fup_t(fname: &str, val: String, fmt: TextFormat, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::Formatted(val, fmt), act)
}

/// Set field with integer
#[inline]
pub fn fset_i(fname: &str, val: i64) -> FieldSetVal {
    fup_i(fname, val, UpdateAction::Null)
}

/// Update field with integer
#[inline]
pub fn fup_i(fname: &str, val: i64, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::Int(val), act)
}

/// Set field with float value
#[inline]
pub fn fset_f(fname: &str, val: f64) -> FieldSetVal {
    fup_f(fname, val, UpdateAction::Null)
}

/// Update field with float value
#[inline]
pub fn fup_f(fname: &str, val: f64, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::Float(val), act)
}

/// Set field to Vec of IDs
#[inline]
pub fn fset_vid(fname: &str, val: Vec<ID>) -> FieldSetVal {
    (fname.to_string(), FieldVal::ArrID(val), UpdateAction::Null)
}

/// Update field to Vec of IDs
#[inline]
pub fn fup_vid(fname: &str, val: Vec<ID>, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::ArrID(val), act)
}

/// Update field to Vec of Strings (names, categories, or uuids)
#[inline]
pub fn fset_vs(fname: &str, val: Vec<String>) -> FieldSetVal {
    fup_vs(fname, val, UpdateAction::Null)
}

/// Update field to Vec of Strings (names, categories, or uuids)
#[inline]
pub fn fup_vs(fname: &str, val: Vec<String>, act: UpdateAction) -> FieldSetVal {
    (fname.to_string(), FieldVal::ArrStr(val), act)
}

/// Hold value of field for set and update operations
#[derive(Debug, PartialEq)]
pub enum FieldVal {
    /// String value - for text and other string fields
    Str(String),
    /// Formatted text
    Formatted(String, TextFormat),
    /// Array of strings (used for people uuids, labels, reference uuids, etc.)
    ArrStr(Vec<String>),
    /// Array of ids (used for people ids, label ids, etc.)
    ArrID(Vec<u64>),
    /// Integer numeric field
    Int(i64),
    /// Float numeric field
    Float(f64),
}

impl fmt::Display for FieldVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FieldVal::{ArrID, ArrStr, Float, Formatted, Int, Str};
        match self {
            Str(s) => write!(f, "{}", s),
            Int(n) => write!(f, "{}", n),
            Float(n) => write!(f, "{}", n),
            Formatted(s, fmt) => write!(f, "({},{})", s, fmt.to_string()),
            ArrStr(arr) => write!(f, "{:?}", arr),
            ArrID(arr) => write!(f, "{:?}", arr),
        }
    }
}
