use std::collections::VecDeque;
use std::io::{Read, Seek};

use arrow_format::ipc;

use crate::array::StructArray;
use crate::datatypes::DataType;
use crate::error::Result;

use super::super::super::IpcField;
use super::super::deserialize::{read, skip, Node};
use super::super::read_basic::*;
use super::super::Dictionaries;

#[allow(clippy::too_many_arguments)]
pub fn read_struct<R: Read + Seek>(
    field_nodes: &mut VecDeque<Node>,
    data_type: DataType,
    ipc_field: &IpcField,
    buffers: &mut VecDeque<&ipc::Schema::Buffer>,
    reader: &mut R,
    dictionaries: &Dictionaries,
    block_offset: u64,
    is_little_endian: bool,
    compression: Option<ipc::Message::BodyCompression>,
    version: ipc::Schema::MetadataVersion,
) -> Result<StructArray> {
    let field_node = field_nodes.pop_front().unwrap();

    let validity = read_validity(
        buffers,
        field_node,
        reader,
        block_offset,
        is_little_endian,
        compression,
    )?;

    let fields = StructArray::get_fields(&data_type);

    let values = fields
        .iter()
        .zip(ipc_field.fields.iter())
        .map(|(field, ipc_field)| {
            read(
                field_nodes,
                field,
                ipc_field,
                buffers,
                reader,
                dictionaries,
                block_offset,
                is_little_endian,
                compression,
                version,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(StructArray::from_data(data_type, values, validity))
}

pub fn skip_struct(
    field_nodes: &mut VecDeque<Node>,
    data_type: &DataType,
    buffers: &mut VecDeque<&ipc::Schema::Buffer>,
) {
    let _ = field_nodes.pop_front().unwrap();

    let _ = buffers.pop_front().unwrap();

    let fields = StructArray::get_fields(data_type);

    fields
        .iter()
        .for_each(|field| skip(field_nodes, field.data_type(), buffers))
}
