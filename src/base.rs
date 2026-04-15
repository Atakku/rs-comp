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
  pub fn parent(&self) -> Option<Base> {
    match self {
      Base::Common => None,

      Base::Pcores => Some(Base::Common),
      Base::Ecores => Some(Base::Common),
      Base::WithGpu => Some(Base::Pcores),
    }
  }

  pub fn apply(&self, c: &mut Service) {
    if let Some(parent) = self.parent() {
      parent.apply(c);
    }

    let host = hostname::get().unwrap();
    let host = host.to_str().unwrap();

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
          "srvr" | "home" => c.add_env("TZ", "Europe/Belgrade"),
          "neko" => c.add_env("TZ", "Europe/Moscow"),
          "fsmp" | "carp" => c.add_env("TZ", "Europe/Berlin"),
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
      Base::WithGpu => match host {
        "srvr" => c.set_string("runtime", "nvidia"),
        "home" => c.set_string("runtime", "amd"),
        _ => {}
      },
    }
  }
}
