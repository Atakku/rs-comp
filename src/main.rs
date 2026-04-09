// Copyright 2026 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::fs;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::value::Value;

use crate::base::Base;
use crate::service::Service;

pub mod base;
pub mod service;

#[derive(Serialize, Deserialize, Clone)]
struct ComposeFile {
  #[serde(default)]
  include: Vec<String>,
  #[serde(default)]
  services: HashMap<String, Service>,
  // temp
  #[serde(default)]
  networks: HashMap<String, HashMap<String, Value>>,
}

impl ComposeFile {
  fn load(files: &mut HashMap<PathBuf, ComposeFile>, root: PathBuf) {
    let path = root.join(".compose.yml");
    println!("LOAD: {}", path.to_str().unwrap());
    let data = fs::read_to_string(path).unwrap();
    let compose = serde_yaml::from_str::<ComposeFile>(&data).unwrap();

    for file in &compose.include {
      Self::load(files, root.join(format!("{file}/")));
    }

    files.insert(root, compose);
  }

  fn link(&self, idmap: &HashMap<String, PathBuf>, files: &mut HashMap<PathBuf, ComposeFile>) {
    for (id, s) in &self.services {
      for (link, hard) in &s.links {
        let path = idmap.get(link).unwrap();
        // Link network
        let target = files.get_mut(path).unwrap().services.get_mut(link).unwrap();
        target.add_net(id, Default::default());
        if *hard {
          target.vec_push("depends_on", id);
        }
      }

      if let Some(_) = &s.postgres {
        let pgid = &format!("{id}_pg");
        let path = idmap.get("srvr_adminer").unwrap();
        // Link network
        let target = files
          .get_mut(path)
          .unwrap()
          .services
          .get_mut("srvr_adminer")
          .unwrap();
        target.add_net(pgid, Default::default());
      }

      if let Some(nginx) = &s.nginx {
        let fallback = &format!("{}_nginx", id.split_once("_").unwrap().0);
        let nginx_id = nginx.instance.clone().unwrap_or(fallback.into());

        let path = idmap.get(&nginx_id).unwrap();
        let target = files
          .get_mut(path)
          .unwrap()
          .services
          .get_mut(&nginx_id)
          .unwrap();
        target.add_net(id, Default::default());
      }
    }
  }

  fn build(&mut self) {
    for path in &mut self.include {
      *path = format!("{path}/.compose.aku.yml");
    }

    for id in self.services.clone().keys() {
      let s = self.services.get_mut(id).unwrap();
      s.apply_base(id);

      if !s.inner.contains_key("network_mode") {
        // Establish container network
        let net = self.networks.entry(id.into()).or_default();
        net.insert("name".into(), Value::String(id.into()));
        net.insert("internal".into(), Value::Bool(s.internal.unwrap_or(false)));

        // Add this network to the container
        s.add_net(id, Default::default());
      }

      if let Some(ngx) = s.nginx.clone() {
        let fallback = &format!("{}_nginx", id.split_once("_").unwrap().0);
        let nginx_id = ngx.instance.clone().unwrap_or(fallback.into());

        s.get_map("labels").insert(
          Value::String(nginx_id.into()),
          Value::String(format!(
            "\
server {{
  include /etc/nginx/shared/ssl.conf;
  server_name {};

  location / {{
    include /etc/nginx/shared/proxy.conf;
    set $$upstream {};
    proxy_pass http://$$upstream:{};
  }}
}}",
            ngx.domain, id, ngx.port
          )),
        );
      }

      // Define postgres container
      if let Some(pgs) = s.postgres.clone() {
        let pgid = &format!("{id}_pg");

        // Make base container
        let mut pg = Service {
          base: Some(Base::Ecores),
          ..Default::default()
        };
        pg.apply_base(pgid);

        // Establish pg network
        let net = self.networks.entry(pgid.into()).or_default();
        net.insert("name".into(), Value::String(pgid.into()));
        net.insert("internal".into(), Value::Bool(true));

        // Add it to parent and self and depencency
        pg.add_net(pgid, Default::default());
        s.add_net(pgid, Default::default());
        s.vec_push("depends_on", pgid);

        // Setup the postgres container
        pg.set_string("image", pgs.image.clone());
        pg.add_env("POSTGRES_DB", id);
        pg.add_env("POSTGRES_USER", id);
        pg.add_env("POSTGRES_PASSWORD", pgs.password.clone());

        pg.vec_push("volumes", format!("{}:/var/lib/postgresql", pgs.path));

        self.services.insert(pgid.into(), pg);
      }
    }
  }

  fn write(self, path: PathBuf) {
    fs::write(
      path.join(".compose.aku.yml"),
      serde_yaml::to_string(&self).unwrap(),
    )
    .unwrap();
  }
}

fn main() {
  let mut files = HashMap::<PathBuf, ComposeFile>::new();
  let root = Path::new(std::env::args().collect::<Vec<_>>().get(1).unwrap()).to_path_buf();

  ComposeFile::load(&mut files, root);

  let idmap = files
    .clone()
    .into_iter()
    .map(|(path, file)| {
      file
        .services
        .into_iter()
        .map(|(id, _)| (id, path.clone()))
        .collect::<Vec<(String, PathBuf)>>()
    })
    .flatten()
    .collect::<HashMap<String, PathBuf>>();

  // Link services
  for (path, file) in files.clone() {
    println!("LINK: {}", path.to_str().unwrap());
    file.link(&idmap, &mut files);
  }

  // Build services
  for (path, file) in &mut files {
    println!("BUILD: {}", path.to_str().unwrap());
    file.build();
  }

  // Write files
  for (path, file) in files {
    println!("WRITE: {}", path.to_str().unwrap());
    file.write(path);
  }
}
