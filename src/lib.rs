#[macro_use]
extern crate slog;
extern crate slog_scope;
#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;
extern crate regex;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate smartfox as smartfox_c;

pub use serde_json::ser::to_string as to_json;
use serde_json::from_str;
use serde_json::value::{Value, from_value};

use data::DATAMGR;
use error::ResultExt;

/// Error
pub mod error;
/// Packet
pub mod packet;
/// Data
pub mod data;
/// Data extractors
pub mod data_extractors;
/// Smartfoxserver client
pub mod smartfox;
/// Goodgame empire connection
pub mod connection;

mod byte_stream_splitter;

/// Read castles
pub fn read_castles(data: data_extractors::gbd::Gbd) {
    for ain in data.ain {
        for castle in ain.ap {
            DATAMGR.lock().unwrap().add_castle(castle);
        }
        for castle in ain.vp {
            DATAMGR.lock().unwrap().add_castle(castle);
        }
    }
}

pub fn read_names(data: String) -> error::Result<()> {
    let data = data.trim_right_matches('%');
    let data: Value = from_str(data).chain_err(
        || "Cant parse json in gge::read_names",
    )?;
    let data = data.as_object().ok_or(error::ErrorKind::InvalidFormat(
        "Root not a object in gge::read_names"
            .into(),
    ))?;
    let gcl = data.get("gcl").unwrap().as_object().unwrap(); // gcl
    let c = gcl.get("C").unwrap().as_array().unwrap(); // gcl C
    for world in c.iter() {
        let world = world.as_object().unwrap();
        let world_name: data::World = from_value(world.get("KID").unwrap().clone()).unwrap(); // gcl C [] KID
        let world = world.get("AI").unwrap().as_array().unwrap(); // gcl C [] AI
        for castle in world {
            let castle = castle
                .as_object()
                .unwrap()
                .get("AI")
                .unwrap()
                .as_array()
                .unwrap(); // gcl C [] AI [] AI (castle)
            let castle_id = castle[3].as_u64().unwrap(); // gcl C [] AI [] AI [3] (id)
            println!("castle: {}", Value::Array(castle.to_owned()));
            let castle_name = get_name_from_slice(castle).expect("Invalid gcl C [] AI [] AI"); // gcl C [] AI [] AI [10] (name)

            let castle = data::Castle {
                id: castle_id,
                owner_id: None,
                name: Some(castle_name.to_string()),
                x: None,
                y: None,
                world: Some(world_name),
            };
            trace!(slog_scope::logger(), "processed castle";  "castle" => format!("{:?}", castle));
            DATAMGR.lock().unwrap().add_castle(castle);
        }
    }
    Ok(())
}

fn get_name_from_slice(slice: &[Value]) -> Option<String> {
    let mut name = None;
    for field in slice.iter() {
        if let &Value::String(ref the_name) = field {
            info!(::slog_scope::logger(), "Found name");
            if name.is_some() {
                error!(::slog_scope::logger(), "Duplicate name {} {}", name.unwrap(), the_name);
            }
            name = Some(the_name.to_string());
        }
    }
    name
}
