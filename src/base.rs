// Copyright 2026 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use serde::Deserialize;

use crate::service::Service;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Base {
  Common,
  Pcores,
  Ecores,
  WithGpu,
}

impl Base {
  pub fn apply(&self, c: &mut Service) {
    match self {
      Base::Pcores => Base::Common.apply(c),
      Base::Ecores => Base::Common.apply(c),
      Base::WithGpu => Base::Pcores.apply(c),
      Base::Common => {}
    };

    match self {
      Base::Common => {
        c.set_string("restart", "always");
        c.set_string("mem_limit", "16G");
        c.add_env("UID", "1000");
        c.add_env("GID", "1000");
        c.add_env("PUID", "1000");
        c.add_env("PGID", "1000");
        c.add_env("TZ", "Europe/Belgrade");
        c.vec_push("volumes", "/etc/localtime:/etc/localtime:ro");
      }
      Base::Pcores => {
        c.set_string("cpuset", "0-15");
      }
      Base::Ecores => {
        c.set_string("cpuset", "16-23");
      }
      Base::WithGpu => {
        c.set_string("runtime", "nvidia");
      }
    }
  }
}