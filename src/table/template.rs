use crate::table::descriptor::{ID, Descriptor, Fxy};
use crate::table::table::{TableGroupManager, TableGroupId, TableGroup};
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

#[derive(Debug)]
pub struct Node {
    pub id: ID,
    pub parent: RefCell<Weak<Node>>,
    pub children: RefCell<Vec<Rc<Node>>>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id.as_string())
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
            id: 0,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        });
        let id_supplier = &mut unexpanded_descriptors.iter().peekable();
        while id_supplier.peek().is_some() {
            let node = create_template_node(table_group, &root, id_supplier)?;
            root.children.borrow_mut().push(node);
        }
        Ok(Template {
            ids: unexpanded_descriptors.to_owned(),
            table_group_id: table_group.id().clone(),
            root,
        })
    }
}

pub fn create_template_node(table_group: &TableGroup,
                            parent: &Rc<Node>,
                            id_supplier: &mut Peekable<Iter<ID>>) -> Result<Rc<Node>, BufrKitError> {
    let id = *id_supplier.next()
        .ok_or(BufrKitError { message: format!("insufficient IDs") })?;
    let node = Rc::new(Node {
        id,
        parent: RefCell::new(Rc::downgrade(parent)),
        children: RefCell::new(vec![]),
    });

    match table_group.lookup(id)? {
        Descriptor::Replication(descriptor) => {
            let n_members = if descriptor.y() == 0 { descriptor.x() + 1 } else { descriptor.x() };
            let mut member_ids = Vec::new();
            for _ in 0..n_members {
                member_ids.push(
                    *id_supplier.next()
                        .ok_or(BufrKitError { message: format!("insufficient IDs") })?);
            }
            expand_members(table_group, &node, member_ids)?;
        }
        Descriptor::Sequence(descriptor) => {
            let mut member_ids = Vec::new();
            for s in descriptor.entry.members.iter() {
                member_ids.push(s.parse::<isize>()?);
            }
            expand_members(table_group, &node, member_ids)?;
        }
        _ => {}
    };
    Ok(node)
}

fn expand_members(table_group: &TableGroup,
                  node: &Rc<Node>,
                  member_ids: Vec<ID>) -> Result<(), BufrKitError> {
    let member_id_supplier = &mut member_ids.iter().peekable();
    while member_id_supplier.peek().is_some() {
        let member_node = create_template_node(
            table_group, &node, member_id_supplier,
        )?;
        node.children.borrow_mut().push(member_node);
    };
    Ok(())
}