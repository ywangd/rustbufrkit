use std::fmt::{Display, Formatter, Error, Result};
use crate::table::table::{BEntry, DEntry};

pub trait Fxy {
    fn id(&self) -> isize;
    fn f(&self) -> isize {
        self.id() / 100_000
    }
    fn x(&self) -> isize {
        (self.id() / 1000) % 100
    }
    fn y(&self) -> isize {
        self.id() % 1000
    }
    fn fx(&self) -> isize {
        self.id() / 1000
    }
    fn as_string(&self) -> String {
        return format!("{:06}", self.id())
    }
}

pub type ID = isize;

impl Fxy for ID {
    fn id(&self) -> isize {
        *self
    }
}

pub enum Descriptor<'a> {
    Element(ElementDescriptor<'a>),
    Replication(ReplicationDescriptor),
    Operator(OperatorDescriptor),
    Sequence(SequenceDescriptor<'a>),
}

impl<'a> Fxy for Descriptor<'a> {
    fn id(&self) -> ID {
        match self {
            Descriptor::Element(d) => d.id,
            Descriptor::Replication(d) => d.id,
            Descriptor::Operator(d) => d.id,
            Descriptor::Sequence(d) => d.id,
        }
    }
}

impl<'a> Display for Descriptor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Descriptor::Element(d) => d.fmt(f),
            _ => write!(f, "{}", self.id().as_string()),
        }
    }
}

pub struct ElementDescriptor<'a>{
    pub id: ID,
    pub entry: &'a BEntry,
}

impl<'a> Display for ElementDescriptor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {:?}", self.id.as_string(), self.entry)
    }
}

pub struct ReplicationDescriptor {
    pub id: ID,
}

impl Fxy for ReplicationDescriptor {
    fn id(&self) -> isize {
        self.id
    }
}

pub struct OperatorDescriptor {
    pub id: ID,
}

pub struct SequenceDescriptor<'a> {
    pub id: ID,
    pub entry: &'a DEntry,
}