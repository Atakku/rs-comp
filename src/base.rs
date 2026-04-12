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
  Gpu,
}

impl Base {
  pub fn parent(&self) -> Option<Base> {
    match self {
      Base::Common => None,

      Base::Pcores => Some(Base::Common),
      Base::Ecores => Some(Base::Common),
      Base::Gpu => Some(Base::Common),
    }
  }

  pub fn apply(&self, c: &mut Service, host: &str) {
    if let Some(parent) = self.parent() {
      parent.apply(c, host);
    }

    match self {
      Base::Common => {
        c.set_string("restart", "always");
        c.set_string("mem_limit", "16G");
        c.add_env("UID", "1000");
        c.add_env("GID", "1000");
        c.add_env("PUID", "1000");
        c.add_env("PGID", "1000");
        c.vec_push("volumes", "/etc/localtime:/etc/localtime:ro");

        match host {
          "srvr" | "home" => {
            c.add_env("TZ", "Europe/Belgrade");
          }
          "neko" => {
            c.add_env("TZ", "Europe/Moscow");
          }
          "fsmp" | "carp" => {
            c.add_env("TZ", "Europe/Berlin");
          }
          _ => {}
        }
      }
      Base::Pcores => match host {
        "srvr" => {
          c.set_string("cpuset", "0-15");
        }
        _ => {}
      },
      Base::Ecores => match host {
        "srvr" => {
          c.set_string("cpuset", "16-23");
        }
        _ => {}
      },
      Base::Gpu => match host {
        "srvr" => {
          c.set_string("cpuset", "0-15");
          c.set_string("runtime", "nvidia");
        }
        "home" => {
          c.set_string("runtime", "amd");
        }
        _ => {}
      },
    }
  }
}
