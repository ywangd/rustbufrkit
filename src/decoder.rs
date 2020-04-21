use std::io::BufRead;
use crate::bufr::{BufrMessage, BufrSection, Field, FieldSimple, FieldUED, SimpleData};
use crate::BufrKitError;
use bitreader::{BitReader, BitReaderError};
use std::borrow::{Borrow, BorrowMut};
use crate::table::table::{TableGroupManager, TableGroupId, TableGroup};
use crate::table::template::{Template, PrintVisitor};

pub fn decode_binary(table_group_manager: &TableGroupManager,
                     r: &mut dyn BufRead) -> Result<BufrMessage, BufrKitError> {
    let bytes = prepare(r)?;
    let br = BitReader::new(bytes.borrow());
    let mut bd = BinaryDecoder {
        table_group_manager,
        br,
    };
    let sections = bd.decode()?;
    Ok(BufrMessage::new(sections))
}

fn prepare(r: &mut dyn BufRead) -> Result<Vec<u8>, BufrKitError> {
    let start_signature = "BUFR".to_string();
    let pos = find_string(&start_signature, r)?;
    let mut bytes = start_signature.as_bytes().to_vec();

    let mut b = [0u8; 3];
    read_bytes(r, &mut b)?;

    let mut bit_reader = BitReader::new(&b[..]);
    let length = bit_reader.read_u32(24)?;
    bytes.extend(&b);

    let mut remaining = vec![0u8; (length - 7) as usize];
    read_bytes(r, remaining.borrow_mut())?;
    bytes.extend(remaining);
    Ok(bytes)
}

pub trait FieldReader {
    fn read_field_bytes(&mut self, name: &str, nbytes: usize) -> Result<Field, BufrKitError>;
    fn read_field_u32(&mut self, name: &str, nbits: u8) -> Result<Field, BufrKitError>;
    fn read_field_bool(&mut self, name: &str) -> Result<Field, BufrKitError>;
    fn read_field_flag(&mut self, name: &str, nbits: u8) -> Result<Field, BufrKitError>;
    fn read_field_raw(&mut self, name: &str, nbits: usize) -> Result<Field, BufrKitError>;
    fn read_field_ued(&mut self, name: &str, n: usize) -> Result<Field, BufrKitError>;
    fn read_field_payload(&mut self, name: &str, nbits: usize, template: &Template) -> Result<Field, BufrKitError>;
}

struct BinaryDecoder<'a> {
    table_group_manager: &'a TableGroupManager,
    br: BitReader<'a>,
}

impl<'a> FieldReader for BinaryDecoder<'a> {
    fn read_field_bytes(&mut self, name: &str, nbytes: usize) -> Result<Field, BufrKitError> {
        let mut b = vec![0u8; nbytes];
        self.br.read_u8_slice(b.borrow_mut())?;
        let v = SimpleData::BYTES(b.iter().map(|&x| x as char).collect());
        Ok(Field::SIMPLE(FieldSimple::new(name, v)))
    }

    fn read_field_u32(&mut self, name: &str, nbits: u8) -> Result<Field, BufrKitError> {
        let v = SimpleData::U32(self.br.read_u32(nbits)?);
        Ok(Field::SIMPLE(FieldSimple::new(name, v)))
    }

    fn read_field_bool(&mut self, name: &str) -> Result<Field, BufrKitError> {
        let v = SimpleData::BOOL(self.br.read_bool()?);
        Ok(Field::SIMPLE(FieldSimple::new(name, v)))
    }

    fn read_field_flag(&mut self, name: &str, nbits: u8) -> Result<Field, BufrKitError> {
        let v = SimpleData::FLAG(self.br.read_u32(nbits)?, nbits);
        Ok(Field::SIMPLE(FieldSimple::new(name, v)))
    }

    fn read_field_raw(&mut self, name: &str, nbits: usize) -> Result<Field, BufrKitError> {
        let mut b = vec![0u8; nbits / 8];
        self.br.read_u8_slice(b.borrow_mut())?;
        let n = nbits % 8;
        if n != 0 {
            b.push(self.br.read_u8(n as u8)?);
        }
        Ok(Field::SIMPLE(FieldSimple::new(name, SimpleData::RAW(b, nbits))))
    }

    fn read_field_ued(&mut self, name: &str, n: usize) -> Result<Field, BufrKitError> {
        let mut ids = Vec::new();
        for _ in 0..n {
            ids.push((self.br.read_u32(2)? * 100_000
                + self.br.read_u32(6)? * 1000
                + self.br.read_u32(8)?) as isize);
        }
        Ok(Field::UED(FieldUED::new(name, ids)))
    }

    fn read_field_payload(&mut self, name: &str, nbits: usize, template: &Template) -> Result<Field, BufrKitError> {
        Err(BufrKitError { message: "".to_owned() })
    }
}

impl<'a> BinaryDecoder<'a> {
    fn decode(&mut self) -> Result<Vec<BufrSection>, BufrKitError> {
        let mut sections = Vec::new();
        self.decode_section_0(&mut sections)?;
        self.decode_section_1(&mut sections)?;
        self.decode_section_2(&mut sections)?;
        self.decode_section_3(&mut sections)?;
        self.decode_section_4(&mut sections)?;
        self.decode_section_5(&mut sections)?;
        Ok(sections)
    }

    fn decode_section_0(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        let mut fields = vec!(
            self.read_field_bytes("start_signature", 4)?,
            self.read_field_u32("length", 24)?,
        );
        let field = self.read_field_u32("edition", 8)?;
        return if field.get_u32() != 4 {
            Err(BufrKitError {
                message: format!("Only support BUFR edition 4, got {}", field.get_u32())
            })
        } else {
            fields.push(field);
            sections.push(BufrSection::new(0, fields));
            Ok(())
        };
    }

    fn decode_section_1(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        let edition = sections[0].field_by_name("edition").unwrap().get_u32();
        match edition {
            1 => Ok(sections.push(BufrSection::new(1, vec!(
                self.read_field_u32("originating_centre", 16)?,
                self.read_field_u32("update_sequence_number", 8)?,
                self.read_field_bool("is_section2_presents")?,
                self.read_field_flag("flag_bits", 7)?,
                self.read_field_u32("data_category", 8)?,
                self.read_field_u32("data_local_subcategory", 8)?,
                self.read_field_u32("master_table_version", 8)?,
                self.read_field_u32("local_table_version", 8)?,
                self.read_field_u32("year", 8)?,
                self.read_field_u32("month", 8)?,
                self.read_field_u32("day", 8)?,
                self.read_field_u32("hour", 8)?,
                self.read_field_u32("minute", 8)?,
                self.read_field_u32("second", 8)?,
            )))),
            2 => Ok(sections.push(BufrSection::new(1, vec!(
                self.read_field_u32("section_length", 24)?,
                self.read_field_u32("master_table_number", 8)?,
                self.read_field_u32("originating_centre", 16)?,
                self.read_field_u32("update_sequence_number", 8)?,
                self.read_field_bool("is_section2_presents")?,
                self.read_field_flag("flag_bits", 7)?,
                self.read_field_u32("data_category", 8)?,
                self.read_field_u32("data_local_subcategory", 8)?,
                self.read_field_u32("master_table_version", 8)?,
                self.read_field_u32("local_table_version", 8)?,
                self.read_field_u32("year", 8)?,
                self.read_field_u32("month", 8)?,
                self.read_field_u32("day", 8)?,
                self.read_field_u32("hour", 8)?,
                self.read_field_u32("minute", 8)?,
                self.read_field_u32("second", 8)?,
            )))),
            3 => Ok(sections.push(BufrSection::new(1, vec!(
                self.read_field_u32("section_length", 24)?,
                self.read_field_u32("master_table_number", 8)?,
                self.read_field_u32("originating_subcentre", 8)?,
                self.read_field_u32("originating_centre", 8)?,
                self.read_field_u32("update_sequence_number", 8)?,
                self.read_field_bool("is_section2_presents")?,
                self.read_field_flag("flag_bits", 7)?,
                self.read_field_u32("data_category", 8)?,
                self.read_field_u32("data_i18n_subcategory", 8)?,
                self.read_field_u32("data_local_subcategory", 8)?,
                self.read_field_u32("master_table_version", 8)?,
                self.read_field_u32("local_table_version", 8)?,
                self.read_field_u32("year", 8)?,
                self.read_field_u32("month", 8)?,
                self.read_field_u32("day", 8)?,
                self.read_field_u32("hour", 8)?,
                self.read_field_u32("minute", 8)?,
                self.read_field_u32("second", 8)?,
            )))),
            4 => Ok(sections.push(BufrSection::new(1, vec!(
                self.read_field_u32("section_length", 24)?,
                self.read_field_u32("master_table_number", 8)?,
                self.read_field_u32("originating_centre", 16)?,
                self.read_field_u32("originating_subcentre", 16)?,
                self.read_field_u32("update_sequence_number", 8)?,
                self.read_field_bool("is_section2_presents")?,
                self.read_field_flag("flag_bits", 7)?,
                self.read_field_u32("data_category", 8)?,
                self.read_field_u32("data_i18n_subcategory", 8)?,
                self.read_field_u32("data_local_subcategory", 8)?,
                self.read_field_u32("master_table_version", 8)?,
                self.read_field_u32("local_table_version", 8)?,
                self.read_field_u32("year", 16)?,
                self.read_field_u32("month", 8)?,
                self.read_field_u32("day", 8)?,
                self.read_field_u32("hour", 8)?,
                self.read_field_u32("minute", 8)?,
                self.read_field_u32("second", 8)?,
            )))),
            _ => Err(BufrKitError {
                message: format!("Unknown BUFR edition number: {}", edition)
            })
        }
    }

    fn decode_section_2(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        if sections[1].field_by_name("is_section2_presents").unwrap().get_bool() {
            let field = self.read_field_u32("section_length", 24)?;
            let n_local_bits = ((field.get_u32() - 4) * 8) as usize;
            sections.push(BufrSection::new(2, vec!(
                field,
                self.read_field_flag("reserved_bits", 8)?,
                self.read_field_raw("local_bits", n_local_bits)?,
            )));
        } else {
            sections.push(BufrSection::new(2, vec!()));
        }
        Ok(())
    }

    fn decode_section_3(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        let field = self.read_field_u32("section_length", 24)?;
        let mut n_descriptors = ((field.get_u32() - 7) / 2) as usize;
        let mut fields = vec!(field);
        fields.push(self.read_field_flag("reserved_bits", 8)?);
        let field = self.read_field_u32("n_subsets", 16)?;
        let n_subsets = field.get_u32();
        fields.push(field);
        fields.push(self.read_field_bool("is_observation")?);
        fields.push(self.read_field_bool("is_compressed")?);
        fields.push(self.read_field_flag("reserved_bits", 6)?);

        fields.push(self.read_field_ued("unexpanded_descriptors", n_descriptors)?);
        sections.push(BufrSection::new(3, fields));
        // TODO: consume padding data if any
        Ok(())
    }

    fn decode_section_4(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        let field = self.read_field_u32("section_length", 24)?;
        let n_data_bits = ((field.get_u32() - 4) * 8) as usize;
        let mut fields = vec!(field);
        fields.push(self.read_field_flag("reserved_bits", 8)?);

        let section_1 = sections.get(1).unwrap();
        let table_group_id = TableGroupId {
            base_dir: "_definitions/tables".to_owned(),
            master_table_number: section_1.field_by_name("master_table_number").unwrap().get_u32() as isize,
            centre_number: section_1.field_by_name("originating_centre").unwrap().get_u32() as isize,
            sub_centre_number: section_1.field_by_name("originating_subcentre").map_or_else(|| 0, |f| f.get_u32()) as isize,
            version_number: section_1.field_by_name("master_table_version").unwrap().get_u32() as isize,
        };
        let table_group = self.table_group_manager.get_table_group(&table_group_id)?;
        let section_3 = sections.get(3).unwrap();
        let unexpanded_descriptors = section_3
            .field_by_name("unexpanded_descriptors")
            .unwrap()
            .get_unexpanded_descriptors();
        let mut template = Template::new(&table_group, unexpanded_descriptors)?;

        fields.push(self.read_field_raw("template_data", n_data_bits)?);
        sections.push(BufrSection::new(4, fields));
        Ok(())
    }

    fn decode_section_5(&mut self, sections: &mut Vec<BufrSection>) -> Result<(), BufrKitError> {
        let field = self.read_field_bytes("stop_signature", 4)?;
        return if field.get_bytes() != "7777" {
            Err(BufrKitError {
                message: format!("Stop signature expected, found: {}", field.get_bytes())
            })
        } else {
            sections.push(BufrSection::new(5, vec!(field)));
            Ok(())
        };
    }
}

pub fn find_string(s: &str, r: &mut dyn BufRead) -> Result<usize, BufrKitError> {
    let states = s.as_bytes();
    let mut i = 0usize;
    let mut b = [0u8];
    let mut p = 0usize;

    while i < states.len() {
        read_bytes(r, &mut b)?;
        i = if b[0] == states[i] { i + 1 } else { 0 };
        p += 1;
    }
    Ok(p - states.len())
}

fn read_bytes(r: &mut dyn BufRead, b: &mut [u8]) -> Result<(), BufrKitError> {
    Ok(r.read_exact(b)?)
}