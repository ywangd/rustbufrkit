use std::collections::HashMap;
use std::fmt;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use crate::table::descriptor::{ID, Fxy};
use std::fs::File;
use crate::BufrKitError;
use std::sync::{RwLock, Arc};

pub enum Entry<'a> {
    B(&'a BEntry),
    R(REntry),
    C(&'a CEntry),
    D(&'a DEntry),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BEntry {
    pub name: String,
    pub unit: String,
    pub scale: isize,
    pub refval: isize,
    pub nbits: isize,
    pub unit_crex: String,
    pub scale_crex: isize,
    pub nchars_crex: isize,
}

pub struct REntry {
    id: ID,
}

impl REntry {
    pub fn n_members(&self) -> isize {
        self.id.x()
    }

    pub fn n_repeats(&self) -> isize {
        self.id.y()
    }
}

pub struct CEntry {
    pub name: String,
    pub definition: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DEntry {
    pub name: String,
    pub members: Vec<isize>,
}

pub struct TableGroupManager {
    cache: RwLock<HashMap<TableGroupId, Arc<TableGroup>>>
}

impl TableGroupManager {
    pub fn new() -> Self {
        TableGroupManager {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_table_group(&self, table_group_id: &TableGroupId) -> Result<Arc<TableGroup>, BufrKitError> {
        if !self.cache.read().unwrap().contains_key(table_group_id) {
            let mut cache = self.cache.write().unwrap();
            cache.insert(table_group_id.clone(), Arc::new(TableGroup::load(table_group_id)?));
        }
        Ok(self.cache.read().unwrap().get(&table_group_id).unwrap().clone())
    }

    pub fn size(&self) -> usize {
        self.cache.read().unwrap().len()
    }
}

pub struct TableGroup {
    id: TableGroupId,
    b: TableB,
    d: TableD,
    cnf: CodeAndFlag,
    ma: MetaA,
    mb: MetaB,
    mc: MetaC,
    md: MetaD,
}

impl TableGroup {
    pub fn load(table_group_id: &TableGroupId) -> Result<TableGroup, BufrKitError> {
        let b = TableB::load(&table_group_id)?;
        let d = TableD::load(&table_group_id)?;
        let cnf = CodeAndFlag::load(&table_group_id)?;
        let ma = MetaA::load(&table_group_id)?;
        let mb = MetaB::load(&table_group_id)?;
        let mc = MetaC::load(&table_group_id)?;
        let md = MetaD::load(&table_group_id)?;
        Ok(TableGroup { id: table_group_id.clone(), b, d, cnf, ma, mb, mc, md })
    }

    pub fn id(&self) -> &TableGroupId {
        &self.id
    }

    /// lookup descriptor with the given id
    pub fn lookup(&self, id: ID) -> Result<Entry, BufrKitError> {
        match id.f() {
            0 => Ok(Entry::B(self.b.lookup(id)?)),
            1 => Ok(Entry::R(REntry { id })),
            2 => Ok(Entry::C(self.mc.lookup(id)?)),
            3 => Ok(Entry::D(self.d.lookup(id)?)),
            _ => Err(BufrKitError {
                message: format!("{}: not a valid form of descriptor ID", id.as_string())
            }),
        }
    }

    pub fn lookup_cnf(&self, id: ID, val: isize) -> Result<&str, BufrKitError> {
        if id.f() != 0 {
            Err(BufrKitError {
                message: format!("{}: not a valid element descriptor ID", id.as_string())
            })
        } else {
            self.cnf.lookup(id, val)
        }
    }

    pub fn lookup_meta(&self, id: ID) -> Result<&str, BufrKitError> {
        match id.f() {
            0 => self.mb.lookup(id),
            2 => {
                let centry = self.mc.lookup(id)?;
                Ok(&centry.name)
            }
            3 => self.md.lookup(id),
            _ => Err(BufrKitError {
                message: format!("{}: metadata not found", id.as_string())
            })
        }
    }

    pub fn data_category_of(&self, code: isize) -> Result<&str, BufrKitError> {
        self.ma.lookup(code)
    }
}

impl fmt::Display for TableGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id={}", self.id)
    }
}

// =================================================
#[derive(Serialize, Deserialize, Debug)]
pub struct TableB(HashMap<isize, BEntry>);

impl TableB {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::TableB))?;
        let t = serde_json::from_reader(ins)?;
        Ok(t)
    }

    fn lookup(&self, id: ID) -> Result<&BEntry, BufrKitError> {
        self.0.get(&id).ok_or(BufrKitError {
            message: format!("{} not found", id.as_string())
        })
    }
}

// =================================================
#[derive(Serialize, Deserialize, Debug)]
pub struct TableD(HashMap<isize, DEntry>);

impl TableD {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::TableD))?;
        let content: HashMap<isize, (String, Vec<String>)> = serde_json::from_reader(ins)?;
        let mut t = HashMap::new();
        for (k, v) in content.into_iter() {
            let mut members = Vec::new();
            for vv in v.1.into_iter() {
                members.push(vv.parse::<isize>()?);
            }
            t.insert(k, DEntry {
                name: v.0,
                members,
            });
        }
        Ok(TableD(t))
    }

    fn lookup(&self, id: ID) -> Result<&DEntry, BufrKitError> {
        self.0.get(&id).ok_or(BufrKitError {
            message: format!("{} not found", id.as_string())
        })
    }
}

pub struct CodeAndFlag(HashMap<isize, HashMap<isize, String>>);

impl CodeAndFlag {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::CodeAndFlag))?;
        let content: HashMap<isize, Vec<(isize, String)>> = serde_json::from_reader(ins)?;

        Ok(CodeAndFlag(content.into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect()))
    }

    fn lookup(&self, id: ID, val: isize) -> Result<&str, BufrKitError> {
        if let Some(v1) = self.0.get(&id) {
            if let Some(v2) = v1.get(&val) {
                return Ok(v2);
            }
        }
        Err(BufrKitError {
            message: format!("Entry not found for {} with value {}", id.as_string(), val)
        })
    }
}

pub struct MetaA {
    entries: HashMap<isize, String>,
}

impl MetaA {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::MetaA))?;

        #[derive(Serialize, Deserialize, Debug)]
        struct Content {
            description: String,
            header: (String, String),
            entries: Vec<(String, String)>,
        }

        let content: Content = serde_json::from_reader(ins)?;

        let mut entries = HashMap::new();
        for entry in content.entries.into_iter() {
            if entry.0.contains(" - ") {
                let bounds: Vec<&str> = entry.0.split(" - ").collect();
                for i in bounds[0].parse::<isize>().unwrap()..=bounds[1].parse::<isize>().unwrap() {
                    entries.insert(i, entry.1.clone());
                }
            } else {
                entries.insert(entry.0.parse::<isize>().unwrap(), entry.1);
            }
        }
        Ok(MetaA { entries })
    }

    fn lookup(&self, code: isize) -> Result<&str, BufrKitError> {
        if let Some(entry) = self.entries.get(&code) {
            Ok(entry)
        } else {
            Err(BufrKitError {
                message: format!("{}: data category not found", code)
            })
        }
    }
}

pub struct MetaB {
    entries: HashMap<String, String>,
}

impl MetaB {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::MetaB))?;

        #[derive(Serialize, Deserialize, Debug)]
        struct Content {
            description: String,
            header: (String, String, String, String),
            entries: Vec<(String, String, String, String)>,
        }
        let content: Content = serde_json::from_reader(ins)?;

        let mut entries = HashMap::new();
        for entry in content.entries.into_iter() {
            entries.insert(
                format!("{}{}", entry.0, entry.1),
                format!("{}: {}", entry.2, entry.3),
            );
        }
        Ok(MetaB { entries })
    }

    fn lookup(&self, id: ID) -> Result<&str, BufrKitError> {
        if let Some(entry) = self.entries.get(&format!("{:03}", id.fx())) {
            Ok(entry)
        } else {
            Err(BufrKitError {
                message: format!("{}: metadata not found", id.as_string())
            })
        }
    }
}

pub struct MetaC {
    entries: HashMap<String, CEntry>,
}

impl MetaC {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::MetaC))?;

        #[derive(Serialize, Deserialize, Debug)]
        struct Content {
            description: String,
            header: (String, String, String, String, String),
            entries: Vec<(String, String, String, String, String)>,
        }
        let content: Content = serde_json::from_reader(ins)?;

        let mut entries = HashMap::new();
        for entry in content.entries.into_iter() {
            entries.insert(
                format!("{}{}{}", entry.0, entry.1, entry.2),
                CEntry { name: entry.3, definition: entry.4 },
            );
        }

        Ok(MetaC {
            entries,
        })
    }

    fn lookup(&self, id: ID) -> Result<&CEntry, BufrKitError> {
        if let Some(centry) = self.entries.get(&id.as_string()) {
            Ok(centry)
        } else if let Some(centry) = self.entries.get(&format!("{:03}YYY", id.fx())) {
            Ok(centry)
        } else {
            Err(BufrKitError { message: format!("{} not found", id.as_string()) })
        }
    }
}

pub struct MetaD {
    entries: HashMap<String, String>,
}

impl MetaD {
    fn load(table_group_id: &TableGroupId) -> Result<Self, BufrKitError> {
        let ins = File::open(table_group_id.get_table_file(Name::MetaD))?;

        #[derive(Serialize, Deserialize, Debug)]
        struct Content {
            description: String,
            header: (String, String, String),
            entries: Vec<(String, String, String)>,
        }

        let content: Content = serde_json::from_reader(ins)?;

        let mut entries = HashMap::new();
        for entry in content.entries.into_iter() {
            entries.insert(
                format!("{}{}", entry.0, entry.1),
                entry.2,
            );
        }
        Ok(MetaD { entries })
    }

    fn lookup(&self, id: ID) -> Result<&str, BufrKitError> {
        if let Some(entry) = self.entries.get(&format!("{:03}", id.fx())) {
            Ok(entry)
        } else {
            Err(BufrKitError { message: format!("{} not found", id.as_string()) })
        }
    }
}

#[derive(Debug)]
pub enum Name {
    TableB,
    TableD,
    CodeAndFlag,
    MetaA,
    MetaB,
    MetaC,
    MetaD,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            Name::TableB => "TableB",
            Name::TableD => "TableD",
            Name::CodeAndFlag => "code_and_flag",
            Name::MetaA => "MetaA",
            Name::MetaB => "MetaB",
            Name::MetaC => "MetaC",
            Name::MetaD => "MetaD",
        };
        write!(f, "{}", t)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct TableGroupId {
    pub base_dir: String,
    pub master_table_number: isize,
    pub centre_number: isize,
    pub sub_centre_number: isize,
    pub version_number: isize,
}

impl fmt::Display for TableGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}{}_{}{}{}",
               self.base_dir, MAIN_SEPARATOR,
               self.master_table_number, MAIN_SEPARATOR,
               self.centre_number, self.sub_centre_number, MAIN_SEPARATOR,
               self.version_number)
    }
}

impl TableGroupId {
    fn get_table_file(&self, name: Name) -> PathBuf {
        let filename = format!("{}.{}", name, "json");
        let base = Path::new(&self.base_dir);
        let mut p = base.join(self.master_table_number.to_string())
            .join(format!("{}_{}", self.centre_number, self.sub_centre_number))
            .join(self.version_number.to_string())
            .join(&filename);

        if p.exists() {
            return p;
        }
        p = base.join("common").join(&filename);
        if p.exists() {
            return p;
        }
        base.join(self.master_table_number.to_string())
            .join("0_0")
            .join(self.version_number.to_string())
            .join(&filename)
    }
}