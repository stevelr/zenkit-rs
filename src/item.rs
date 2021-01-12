use crate::{
    types::{Entry, Field, ID, UUID},
    Error,
};
use std::{iter::Iterator, string::ToString};

/// Item in a list. Wrapper for Entry that includes field getters for business values.
#[derive(Debug)]
pub struct Item<'li> {
    entry: Entry,
    list_name: &'li str,
    list_id: ID,
    fields: &'li [Field],
}

impl<'li> Item<'li> {
    /// Constructs new item
    pub(crate) fn new(
        entry: Entry,
        list_name: &'li str,
        list_id: ID,
        fields: &'li [Field],
    ) -> Self {
        Self {
            entry,
            list_name,
            list_id,
            fields,
        }
    }

    /// Returns reference to the inner entry. Also available with deref.
    pub fn as_entry(&self) -> &Entry {
        &self.entry
    }

    /// Return entry id
    pub fn get_id(&self) -> ID {
        self.entry.id
    }

    /// Returns entry uuid
    pub fn get_uuid(&self) -> &UUID {
        &self.entry.uuid
    }

    /// Returns field (definition) given its name, id, or uuid
    pub fn get_field(&self, field_id: &str) -> Result<&Field, Error> {
        self.fields
            .iter()
            .find(|e| e.name == field_id || e.uuid == field_id || e.id.to_string() == field_id)
            .ok_or_else(|| {
                Error::Other(format!(
                    "Invalid field '{}' in list {}",
                    field_id, self.list_name,
                ))
            })
    }

    /// Returns value of text field. or None if not defined
    /// fname parameter may be field name, id, or uuid
    pub fn get_text_value(&self, fname: &str) -> Result<Option<&str>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_text_value(&field.uuid))?
    }

    /// Returns value of integer field as i64. or None if not defined
    /// fname parameter may be field name, id, or uuid
    pub fn get_int_value(&self, fname: &str) -> Result<Option<i64>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_int_value(&field.uuid))?
    }

    /// Returns value of float field. or None if not defined
    /// fname parameter may be field name, id, or uuid
    pub fn get_float_value(&self, fname: &str) -> Result<Option<f64>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_float_value(&field.uuid))?
    }

    /// Returns value of date field. or None if not defined
    /// fname parameter may be field name, id, or uuid
    pub fn get_date_value(&self, fname: &str) -> Result<Option<&str>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_date_value(&field.uuid))?
    }

    /// Returns display names of persons in field value.
    /// fname parameter may be field name, id, or uuid
    pub fn get_person_names(&self, fname: &str) -> Result<Vec<&str>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_person_names(&field.uuid))
    }

    /// Returns IDs of persons in field value.
    /// fname parameter may be field name, id, or uuid
    pub fn get_person_ids(&self, fname: &str) -> Result<Vec<ID>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_person_ids(&field.uuid))
    }

    /// Returns uuids of referred objects in field value.
    /// fname parameter may be field name, id, or uuid
    pub fn get_references(&self, fname: &str) -> Result<Vec<&str>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_references(&field.uuid))
    }

    /// Returns array of choice (aka label/category) values.
    /// Array could be empty if none are selected
    pub fn get_choices(&self, fname: &str) -> Result<Vec<&str>, Error> {
        self.get_field(fname)
            .map(|field| self.entry.get_category_names(&field.uuid))
    }

    /// Returns single choice value, or None if unselected.
    /// This will return an error if multiple values are selected, which would mean
    /// that the programmer believed the field had multiple-choices disabled,
    /// and they were enabled in the UI.
    pub fn get_choice(&self, fname: &str) -> Result<Option<&str>, Error> {
        let names = self.get_choices(fname)?;
        match names.len() {
            0 => Ok(None),
            1 => Ok(Some(names.get(0).unwrap())),
            _ => Err(Error::MultiCategory(
                "Configuration error: label field not expected to contain multiple values"
                    .to_string(),
                fname.to_string(),
            )),
        }
    }
}

impl<'li> std::ops::Deref for Item<'li> {
    type Target = Entry;

    fn deref(&self) -> &Entry {
        &self.entry
    }
}
