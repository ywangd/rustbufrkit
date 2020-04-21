use crate::table::template::{Visitor, Node, Template};
use std::rc::Rc;
use std::cell::Ref;
use bitreader::BitReader;
use crate::table::descriptor::{ElementDescriptor, ReplicationDescriptor, OperatorDescriptor, SequenceDescriptor};
use crate::bufr::SimpleData;

struct BinaryPayloadDecodingVisitor<'a> {
    template: &'a Template,
    br: &'a BitReader<'a>,
    data: Vec<SimpleData>
}

impl<'a> Visitor for BinaryPayloadDecodingVisitor<'a> {
    fn visit_element_descriptor(&mut self, descriptor: &ElementDescriptor) {
        unimplemented!()
    }

    fn visit_replication_descriptor<'b>(&mut self, descriptor: &ReplicationDescriptor, children: Ref<'b, Vec<Rc<Node>>>) {
        unimplemented!()
    }

    fn visit_operator_descriptor(&mut self, descriptor: &OperatorDescriptor) {
        unimplemented!()
    }

    fn visit_sequence_descriptor<'b>(&mut self, descriptor: &SequenceDescriptor, children: Ref<'b, Vec<Rc<Node>>>) {
        unimplemented!()
    }

    fn visit_replication_factor(&mut self, descriptor: &ElementDescriptor) {
        unimplemented!()
    }

    fn exit_replication_descriptor(&mut self) {
        unimplemented!()
    }

    fn exit_sequence_descriptor(&mut self) {
        unimplemented!()
    }
}