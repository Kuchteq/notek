use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use algos::doc::Doc;
use anyhow::{Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

#[derive(Debug)]
pub struct State {
    pub docs: BTreeMap<u64, DocStructure>,
}
impl State {
    pub fn init(dir: &str) -> Result<Self> {
        let mut d = BTreeMap::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {

                let name = path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid UTF-8 path: {:?}", path))?
                    .to_string();

                let structure = DocStructure::init_from_filepath(name)?;
                d.insert(structure.last_modified, structure);

            }
        }
        Ok(State { docs: d })
    }
}


#[derive(Debug)]
pub struct DocStructure {
    pub id: u128,
    last_modified: u64, // todo, change this, this field shouldn't be duplicating the key of the
                        // btree
    name: String,
    state: DocState,
}

impl DocStructure {
    fn init_from_filepath(name: String) -> Result<DocStructure> {
        let structure_path = &format!("{}.structure", name);
        let structure_path = Path::new(structure_path);

        if structure_path.exists() {
            let file = File::open(structure_path)?;
            let mut reader = BufReader::new(file);
            let id = reader.read_u128::<LittleEndian>()?;
            let last_modified = reader.read_u64::<LittleEndian>()?;
            let doc = Doc::from_reader_eof(&mut reader)?;

            return Ok(DocStructure {
                id,
                name,
                last_modified,
                state: DocState::Cached(doc),
            });
        } else {
            let f = File::create(structure_path)?;
            let mut writer = BufWriter::new(f);

            let id: u128 = Uuid::new_v4().as_u128();
            let timestamp_ms: u64 =
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

            writer.write_all(&id.to_le_bytes())?;
            writer.write_all(&timestamp_ms.to_le_bytes())?;

            let doc = Doc::new(fs::read_to_string(&name)?);
            doc.write_bytes(&mut writer)?;
            writer.flush()?;

            return Ok(DocStructure {
                id,
                name,
                last_modified: timestamp_ms,
                state: DocState::Cached(doc),
            });
        }
    }
}

#[derive(Debug)]
enum DocState {
    Missing,
    Cached(Doc),
}
