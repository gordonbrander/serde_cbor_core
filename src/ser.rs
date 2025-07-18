//! Serialization.
#[cfg(not(feature = "std"))]
use alloc::{collections::TryReserveError, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use std::collections::TryReserveError;

pub use cbor4ii::core::utils::BufWriter;
#[cfg(feature = "std")]
use cbor4ii::core::utils::IoWriter;
use cbor4ii::core::{
    enc::{self, Encode},
    types,
};
use serde::Serialize;

use crate::error::EncodeError;

/// Serializes a value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, EncodeError<TryReserveError>>
where
    T: Serialize + ?Sized,
{
    let writer = BufWriter::new(Vec::new());
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    Ok(serializer.into_inner().into_inner())
}

/// Serializes a value to a writer.
#[cfg(feature = "std")]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), EncodeError<std::io::Error>>
where
    W: std::io::Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(IoWriter::new(writer));
    value.serialize(&mut serializer)
}

/// A structure for serializing Rust values to DAG-CBOR.
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W> {
    /// Creates a new CBOR serializer.
    pub fn new(writer: W) -> Serializer<W> {
        Serializer { writer }
    }

    /// Returns the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W: enc::Write> serde::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    type SerializeSeq = CollectSeq<'a, W>;
    type SerializeTuple = BoundedCollect<'a, W>;
    type SerializeTupleStruct = BoundedCollect<'a, W>;
    type SerializeTupleVariant = BoundedCollect<'a, W>;
    type SerializeMap = CollectMap<'a, W>;
    type SerializeStruct = CollectMap<'a, W>;
    type SerializeStructVariant = CollectMap<'a, W>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        // In DAG-CBOR floats are always encoded as f64.
        self.serialize_f64(f64::from(v))
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        // In DAG-CBOR only finite floats are supported.
        if !v.is_finite() {
            Err(EncodeError::Msg(
                "Float must be a finite number, not Infinity or NaN".into(),
            ))
        } else {
            v.encode(&mut self.writer)?;
            Ok(())
        }
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        types::Bytes(v).encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        types::Null.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // The cbor4ii Serde implementation encodes unit as an empty array, for DAG-CBOR we encode
        // it as `NULL`.
        types::Null.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        enc::MapStartBounded(1).encode(&mut self.writer)?;
        variant.encode(&mut self.writer)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        CollectSeq::new(self, len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        enc::ArrayStartBounded(len).encode(&mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        enc::MapStartBounded(1).encode(&mut self.writer)?;
        variant.encode(&mut self.writer)?;
        enc::ArrayStartBounded(len).encode(&mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(CollectMap::new(self))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        enc::MapStartBounded(len).encode(&mut self.writer)?;
        Ok(CollectMap::new(self))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        enc::MapStartBounded(1).encode(&mut self.writer)?;
        variant.encode(&mut self.writer)?;
        enc::MapStartBounded(len).encode(&mut self.writer)?;
        Ok(CollectMap::new(self))
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        if !(u64::MAX as i128 >= v && -(u64::MAX as i128 + 1) <= v) {
            return Err(EncodeError::Msg(
                "Integer must be within [-u64::MAX-1, u64::MAX] range".into(),
            ));
        }

        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        if (u64::MAX as u128) < v {
            return Err(EncodeError::Msg(
                "Unsigned integer must be within [0, u64::MAX] range".into(),
            ));
        }
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

/// Struct for implementign SerializeSeq.
pub struct CollectSeq<'a, W> {
    /// The number of elements. This is used in case the number of elements is not known
    /// beforehand.
    count: usize,
    ser: &'a mut Serializer<W>,
    /// An in-memory serializer in case the number of elements is not known beforehand.
    mem_ser: Option<Serializer<BufWriter>>,
}

impl<'a, W: enc::Write> CollectSeq<'a, W> {
    /// If the length of the sequence is given, use it. Else buffer the sequence in order to count
    /// the number of elements, which is then written before the elements are.
    fn new(ser: &'a mut Serializer<W>, len: Option<usize>) -> Result<Self, EncodeError<W::Error>> {
        let mem_ser = if let Some(len) = len {
            enc::ArrayStartBounded(len).encode(&mut ser.writer)?;
            None
        } else {
            Some(Serializer::new(BufWriter::new(Vec::new())))
        };
        Ok(Self {
            count: 0,
            ser,
            mem_ser,
        })
    }
}

/// Helper for processing collections.
pub struct BoundedCollect<'a, W> {
    ser: &'a mut Serializer<W>,
}

impl<W: enc::Write> serde::ser::SerializeSeq for CollectSeq<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.count += 1;
        if let Some(ser) = self.mem_ser.as_mut() {
            value
                .serialize(&mut *ser)
                .map_err(|_| EncodeError::Msg("List element cannot be serialized".to_string()))
        } else {
            value.serialize(&mut *self.ser)
        }
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Data was buffered in order to be able to write out the number of elements before they
        // are serialized.
        if let Some(ser) = self.mem_ser {
            enc::ArrayStartBounded(self.count).encode(&mut self.ser.writer)?;
            self.ser.writer.push(&ser.into_inner().into_inner())?;
        }

        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTuple for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTupleStruct for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTupleVariant for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// CBOR RFC-8949 specifies a canonical sort order, where keys are sorted in bytewise
/// lexicographic order. We serialize keys and values separately, then sort by key bytes only.
/// Once sorted, the key-value pairs are written to the actual output.
pub struct CollectMap<'a, W> {
    key_buffer: BufWriter,
    value_buffer: BufWriter,
    entries: Vec<(Vec<u8>, Vec<u8>)>, // (key_bytes, value_bytes)
    ser: &'a mut Serializer<W>,
}

impl<'a, W> CollectMap<'a, W>
where
    W: enc::Write,
{
    fn new(ser: &'a mut Serializer<W>) -> Self {
        Self {
            key_buffer: BufWriter::new(Vec::new()),
            value_buffer: BufWriter::new(Vec::new()),
            entries: Vec::new(),
            ser,
        }
    }

    fn serialize<T: Serialize + ?Sized>(
        &mut self,
        maybe_key: Option<&'static str>,
        value: &T,
    ) -> Result<(), EncodeError<W::Error>> {
        // Serialize the key separately
        let key_bytes = if let Some(key) = maybe_key {
            let mut key_serializer = Serializer::new(&mut self.key_buffer);
            key.serialize(&mut key_serializer)
                .map_err(|_| EncodeError::Msg("Struct key cannot be serialized.".to_string()))?;
            let key_bytes = self.key_buffer.buffer().to_vec();
            self.key_buffer.clear();
            key_bytes
        } else {
            Vec::new()
        };

        // Serialize the value separately
        let mut value_serializer = Serializer::new(&mut self.value_buffer);
        value
            .serialize(&mut value_serializer)
            .map_err(|_| EncodeError::Msg("Struct value cannot be serialized.".to_string()))?;
        let value_bytes = self.value_buffer.buffer().to_vec();
        self.value_buffer.clear();

        self.entries.push((key_bytes, value_bytes));

        Ok(())
    }

    fn end(mut self) -> Result<(), EncodeError<W::Error>> {
        // Sort entries by key bytes only in lexicographic order per RFC 8949
        self.entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        for (key_bytes, value_bytes) in self.entries {
            self.ser.writer.push(&key_bytes)?;
            self.ser.writer.push(&value_bytes)?;
        }
        Ok(())
    }
}

impl<W> serde::ser::SerializeMap for CollectMap<'_, W>
where
    W: enc::Write,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        // Serialize the key into the key buffer
        let mut key_serializer = Serializer::new(&mut self.key_buffer);
        key.serialize(&mut key_serializer)
            .map_err(|_| EncodeError::Msg("Map key cannot be serialized.".to_string()))?;
        Ok(())
    }

    #[inline]
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        // Serialize the value into the value buffer
        let mut value_serializer = Serializer::new(&mut self.value_buffer);
        value
            .serialize(&mut value_serializer)
            .map_err(|_| EncodeError::Msg("Map value cannot be serialized.".to_string()))?;

        // Now store both key and value bytes as a pair
        let key_bytes = self.key_buffer.buffer().to_vec();
        let value_bytes = self.value_buffer.buffer().to_vec();
        self.key_buffer.clear();
        self.value_buffer.clear();

        self.entries.push((key_bytes, value_bytes));
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        enc::MapStartBounded(self.entries.len()).encode(&mut self.ser.writer)?;
        self.end()
    }
}

impl<W> serde::ser::SerializeStruct for CollectMap<'_, W>
where
    W: enc::Write,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.serialize(Some(key), value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end()
    }
}

impl<W> serde::ser::SerializeStructVariant for CollectMap<'_, W>
where
    W: enc::Write,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.serialize(Some(key), value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end()
    }
}
