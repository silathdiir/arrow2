use std::collections::VecDeque;
use std::io::{Read, Seek};

use arrow_format::ipc;

use crate::array::UnionArray;
use crate::datatypes::DataType;
use crate::datatypes::UnionMode::Dense;
use crate::error::Result;

use super::super::super::IpcField;
use super::super::deserialize::{read, skip, Node};
use super::super::read_basic::*;
use super::super::Dictionaries;

#[allow(clippy::too_many_arguments)]
pub fn read_union<R: Read + Seek>(
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
) -> Result<UnionArray> {
    let field_node = field_nodes.pop_front().unwrap();

    if version != ipc::Schema::MetadataVersion::V5 {
        let _ = buffers.pop_front().unwrap();
    };

    let types = read_buffer(
        buffers,
        field_node.length() as usize,
        reader,
        block_offset,
        is_little_endian,
        compression,
    )?;

    let offsets = if let DataType::Union(_, _, mode) = data_type {
        if !mode.is_sparse() {
            Some(read_buffer(
                buffers,
                field_node.length() as usize,
                reader,
                block_offset,
                is_little_endian,
                compression,
            )?)
        } else {
            None
        }
    } else {
        panic!()
    };

    let fields = UnionArray::get_fields(&data_type);

    let fields = fields
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

    Ok(UnionArray::from_data(data_type, types, fields, offsets))
}

pub fn skip_union(
    field_nodes: &mut VecDeque<Node>,
    data_type: &DataType,
    buffers: &mut VecDeque<&ipc::Schema::Buffer>,
) {
    let _ = field_nodes.pop_front().unwrap();

    let _ = buffers.pop_front().unwrap();
    if let DataType::Union(_, _, Dense) = data_type {
        let _ = buffers.pop_front().unwrap();
    } else {
        panic!()
    };

    let fields = UnionArray::get_fields(data_type);

    fields
        .iter()
        .for_each(|field| skip(field_nodes, field.data_type(), buffers))
}
