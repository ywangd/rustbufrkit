use crate::table::descriptor::{ID, Descriptor, Fxy, ReplicationDescriptor, SequenceDescriptor, ElementDescriptor, OperatorDescriptor};
use crate::table::table::{TableGroupManager, TableGroupId, TableGroup, Entry};
use crate::BufrKitError;
use std::borrow::{Borrow, BorrowMut};
use std::sync::Arc;
use std::convert::TryInto;
use std::ops::Deref;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fmt;
use serde::export::Formatter;
use std::slice::Iter;
use std::iter::Peekable;
use serde::de::Unexpected::Seq;

#[derive(Debug)]
pub struct Node {
    pub descriptor: Descriptor,
    pub parent: RefCell<Weak<Node>>,
    pub children: RefCell<Vec<Rc<Node>>>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.descriptor)
    }
}

#[derive(Debug)]
pub struct Template {
    ids: Vec<ID>,
    table_group_id: TableGroupId,
    root: Rc<Node>,
}

impl Template {
    pub fn new(table_group: &TableGroup,
               unexpanded_descriptors: &[isize]) -> Result<Template, BufrKitError> {
        let root = Rc::new(Node {
            descriptor: Descriptor::Sequence(SequenceDescriptor { id: 0, name: "ROOT".to_owned() }),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        });
        expand_members(table_group, &root, unexpanded_descriptors.into())?;
        Ok(Template {
            ids: unexpanded_descriptors.to_owned(),
            table_group_id: table_group.id().clone(),
            root,
        })
    }
}

fn expand_members(table_group: &TableGroup,
                  parent: &Rc<Node>,
                  member_ids: Vec<ID>) -> Result<(), BufrKitError> {
    let member_id_supplier = &mut member_ids.iter().peekable();
    while member_id_supplier.peek().is_some() {
        parent.children.borrow_mut().push(expand_one(
            table_group, &parent, member_id_supplier,
        )?);
    };
    Ok(())
}

pub fn expand_one(table_group: &TableGroup,
                  parent: &Rc<Node>,
                  id_supplier: &mut Peekable<Iter<ID>>) -> Result<Rc<Node>, BufrKitError> {
    let id = *id_supplier.next()
        .ok_or(BufrKitError { message: format!("insufficient IDs") })?;

    let (descriptor, member_ids) = match table_group.lookup(id)? {
        Entry::B(bentry) => {
            (Descriptor::Element(ElementDescriptor {
                id,
                name: bentry.name.to_owned(),
                unit: bentry.unit.to_owned(),
                scale: bentry.scale,
                refval: bentry.refval,
                nbits: bentry.nbits,
            }), vec![])
        }
        Entry::C(centry) => {
            (Descriptor::Operator(OperatorDescriptor {
                id,
                name: centry.name.to_owned(),
            }), vec![])
        }
        Entry::D(dentry) => {
            let mut member_ids = Vec::new();
            for s in dentry.members.iter() {
                member_ids.push(*s);
            }

            (Descriptor::Sequence(SequenceDescriptor { id, name: dentry.name.to_owned() }), member_ids)
        }
        Entry::R(rentry) => {
            let n_members = if rentry.n_repeats() == 0 { rentry.n_members() + 1 } else { rentry.n_members() };
            let mut member_ids = Vec::new();
            for _ in 0..n_members {
                member_ids.push(
                    *id_supplier.next()
                        .ok_or(BufrKitError { message: format!("insufficient IDs") })?);
            }
            (Descriptor::Replication(ReplicationDescriptor { id }), member_ids)
        }
    };
    let node = Rc::new(Node {
        descriptor,
        parent: RefCell::new(Rc::downgrade(parent)),
        children: RefCell::new(vec![]),
    });
    if member_ids.len() > 0 {
        expand_members(table_group, &node, member_ids)?;
    }
    Ok(node)
}
