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
        return format!("{:06}", self.id());
    }
}

pub type ID = isize;

impl Fxy for ID {
    fn id(&self) -> isize {
        *self
    }
}

#[derive(Debug)]
pub enum Descriptor {
    Element(ElementDescriptor),
    Replication(ReplicationDescriptor),
    Operator(OperatorDescriptor),
    Sequence(SequenceDescriptor),
}

impl Fxy for Descriptor {
    fn id(&self) -> ID {
        match self {
            Descriptor::Element(d) => d.id,
            Descriptor::Replication(d) => d.id,
            Descriptor::Operator(d) => d.id,
            Descriptor::Sequence(d) => d.id,
        }
    }
}

impl Display for Descriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Descriptor::Element(d) => d.fmt(f),
            Descriptor::Replication(d) => d.fmt(f),
            Descriptor::Operator(d) => d.fmt(f),
            Descriptor::Sequence(d) => d.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct ElementDescriptor {
    pub id: ID,
    pub name: String,
    pub unit: String,
    pub scale: isize,
    pub refval: isize,
    pub nbits: isize,
}

impl Display for ElementDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {}", self.id.as_string(), self.name)
    }
}

#[derive(Debug)]
pub struct ReplicationDescriptor {
    pub id: ID,
}

impl Fxy for ReplicationDescriptor {
    fn id(&self) -> isize {
        self.id
    }
}

impl Display for ReplicationDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.id.as_string())
    }
}

#[derive(Debug)]
pub struct OperatorDescriptor {
    pub id: ID,
    pub name: String,
}

impl Display for OperatorDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {}", self.id.as_string(), self.name)
    }
}

impl Fxy for OperatorDescriptor {
    fn id(&self) -> isize {
        self.id
    }
}

#[derive(Debug)]
pub struct SequenceDescriptor {
    pub id: ID,
    pub name: String,
}

impl Fxy for SequenceDescriptor {
    fn id(&self) -> isize {
        self.id
    }
}

impl Display for SequenceDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {}", self.id.as_string(), self.name)
    }
}
