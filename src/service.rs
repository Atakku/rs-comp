// Copyright 2026 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use crate::base::Base;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Service {
  #[serde(skip_serializing)]
  pub base: Option<Base>,
  #[serde(skip_serializing)]
  pub internal: Option<bool>,
  #[serde(skip_serializing)]
  pub postgres: Option<PostgresSettings>,
  #[serde(skip_serializing)]
  pub nginx: Option<NginxSettings>,
  #[serde(skip_serializing)]
  #[serde(default)]
  pub links: HashMap<String, bool>,

  #[serde(flatten)]
  pub(crate) inner: HashMap<String, Value>,
}

#[derive(Default, Deserialize, Clone)]
pub struct PostgresSettings {
  pub external: Option<String>,
  pub password: String,
  pub image: String,
  pub path: String,
}

#[derive(Default, Deserialize, Clone)]
pub struct NginxSettings {
  pub instance: Option<String>,
  pub domain: String,
  pub port: u16,
  pub extra_inner: Option<String>,
  pub extra_outer: Option<String>
}

impl Service {
  pub fn apply_base(&mut self, id: &str) {
    let Some(base) = self.base.clone() else {
      return;
    };

    // Swap so we can override the base
    let mut old = HashMap::new();
    std::mem::swap(&mut self.inner, &mut old);

    base.apply(self);

    // Name containers appropriately unless overriden
    self.set_string("container_name", id);
    self.set_string("hostname", id.split_once("_").unwrap().0);

    // Merge the two
    for (k, mut v) in old {
      if let Some(i) = self.inner.get_mut(&k) {
        match i {
          Value::Sequence(vals) => vals.append(v.as_sequence_mut().unwrap()),
          Value::Mapping(vals) => vals.extend(v.as_mapping_mut().unwrap().clone()),
          _ => *i = v.clone(),
        }
      } else {
        self.inner.insert(k, v);
      }
    }
  }

  pub fn get_map(&mut self, name: impl Into<String>) -> &mut Mapping {
    self
      .inner
      .entry(name.into())
      .or_insert(Value::Mapping(Default::default()))
      .as_mapping_mut()
      .unwrap()
  }
  pub fn get_vec(&mut self, name: impl Into<String>) -> &mut Vec<Value> {
    self
      .inner
      .entry(name.into())
      .or_insert(Value::Sequence(Default::default()))
      .as_sequence_mut()
      .unwrap()
  }

  pub fn add_net(&mut self, id: impl Into<String>, value: Mapping) {
    self
      .get_map("networks")
      .insert(Value::String(id.into()), Value::Mapping(value));
  }
  pub fn add_env(&mut self, k: impl Into<String>, v: impl Into<String>) {
    self
      .get_map("environment")
      .insert(Value::String(k.into()), Value::String(v.into()));
  }
  pub fn vec_push(&mut self, k: impl Into<String>, v: impl Into<String>) {
    self.get_vec(k).push(Value::String(v.into()));
  }

  pub fn set_string(&mut self, k: impl Into<String>, v: impl Into<String>) {
    self.inner.insert(k.into(), Value::String(v.into()));
  }
}
