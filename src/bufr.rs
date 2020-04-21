use std::any::Any;
use std::collections::HashMap;
use std::iter::FromIterator;
use crate::BufrKitError;

lazy_static! {
    static ref MISSING_BITS_LOOKUP: HashMap<u8, u64> = {
        let mut m = HashMap::new();
        for i in (0..64).rev() {
            m.insert(64 - i, !1u64 >> i);
        }
        m
    };
}

#[derive(Debug)]
pub struct BufrMessage {
    sections: Vec<BufrSection>,
}

impl BufrMessage {
    pub fn new(sections: Vec<BufrSection>) -> BufrMessage {
        BufrMessage {
            sections,
        }
    }

    pub fn length(&self) -> u32 {
        self.sections[0].length()
    }

    pub fn edition(&self) -> u32 {
        self.sections[0].fields[2].get_u32()
    }

    pub fn test(&mut self) {
        println!("missing {}", MISSING_BITS_LOOKUP.get(&64u8).unwrap());
    }
}

#[derive(Debug, Default)]
pub struct BufrSection {
    index: u8,
    fields: Vec<Field>,
    lookup: HashMap<String, usize>,
}

impl BufrSection {
    pub fn new(index: u8, fields: Vec<Field>) -> Self {
        let lookup = fields.iter().enumerate()
            .map(|(i, f)| (f.name().to_owned(), i)).collect();
        BufrSection {
            index,
            fields,
            lookup,
        }
    }

    pub fn length(&self) -> u32 {
        return if self.index == 0 {
            self.fields[1].get_u32()
        } else if self.index == 2 && self.fields.len() == 0 {
            0
        } else {
            self.fields[0].get_u32()
        };
    }

    pub fn field_by_name(&self, name: &str) -> Option<&Field> {
        if let Some(i) = self.lookup.get(name) {
            Some(self.field(*i))
        } else {
            None
        }
    }

    pub fn field(&self, i: usize) -> &Field {
        &self.fields[i]
    }
}


#[derive(Debug)]
pub enum SimpleData {
    U32(u32),
    F64(f64),
    BYTES(String),
    FLAG(u32, u8),
    // value, nbits
    BOOL(bool),
    RAW(Vec<u8>, usize), // value, nbits
}

impl SimpleData {
    pub fn get_u32(&self) -> u32 {
        return if let SimpleData::U32(v) = self {
            *v
        } else {
            panic!("{:?}: cannot get u32 value", self)
        };
    }

    pub fn get_bytes(&self) -> &str {
        return if let SimpleData::BYTES(v) = self {
            &v
        } else {
            panic!("{:?}: cannot get str value", self)
        };
    }

    pub fn get_bool(&self) -> bool {
        return if let SimpleData::BOOL(v) = self {
            *v
        } else {
            panic!("{:?}: cannot get bool value", self)
        };
    }
}

#[derive(Debug)]
pub enum Field {
    SIMPLE(FieldSimple),
    UED(FieldUED),
    PAYLOAD(FieldPayload),
}

impl Field {
    pub fn name(&self) -> &str {
        match self {
            Field::SIMPLE(f) => &f.name,
            Field::UED(f) => &f.name,
            Field::PAYLOAD(f) => &f.name,
        }
    }

    pub fn get_simple_data(&self) -> &SimpleData {
        if let Field::SIMPLE(f) = self {
            &f.value
        } else {
            panic!("{:?}: cannot get simple value", self)
        }
    }

    pub fn get_unexpanded_descriptors(&self) -> &Vec<isize> {
        if let Field::UED(f) = self {
            &f.value
        } else {
            panic!("cannot get unexpanded descriptors: {:?}", self)
        }
    }

    pub fn get_u32(&self) -> u32 {
        self.get_simple_data().get_u32()
    }

    pub fn get_bytes(&self) -> &str {
        self.get_simple_data().get_bytes()
    }

    pub fn get_bool(&self) -> bool {
        self.get_simple_data().get_bool()
    }
}

#[derive(Debug)]
pub struct FieldSimple {
    name: String,
    value: SimpleData,
}

impl FieldSimple {
    pub fn new(name: &str, value: SimpleData) -> Self {
        FieldSimple { name: name.to_owned(), value }
    }
}

#[derive(Debug)]
pub struct FieldUED {
    name: String,
    value: Vec<isize>,
}

impl FieldUED {
    pub fn new(name: &str, value: Vec<isize>) -> Self {
        FieldUED { name: name.to_owned(), value }
    }
}

// Template Data
#[derive(Debug)]
pub struct FieldPayload {
    name: String,
    value: Vec<SimpleData>,
}

impl FieldPayload {
    pub fn new(name: &str, value: Vec<SimpleData>) -> Self {
        FieldPayload { name: name.to_owned(), value }
    }
}
