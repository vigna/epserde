/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Serialization traits and types.

[`Serialize`] is the main serialization trait, providing a
[`Serialize::serialize`] method that serializes the type into a
generic [`WriteNoStd`] backend, and a [`Serialize::serialize_with_schema`] method
that additionally returns a [`Schema`] describing the data that has been written.
The implementation of this trait
is based on [`SerializeInner`], which is automatically derived
with `#[derive(Serialize)]`.

*/

use crate::traits::*;
use crate::*;

use core::hash::Hasher;
use std::{io::BufWriter, path::Path};

pub mod write_with_names;
pub use write_with_names::*;
pub mod helpers;
pub use helpers::*;
pub mod write;
pub use write::*;

pub type Result<T> = core::result::Result<T, Error>;

/// Main serialization trait. It is separated from [`SerializeInner`] to
/// avoid that the user modify its behavior, and hide internal serialization
/// methods.
///
/// It provides a convenience method [`Serialize::store`] that serializes
/// the type to a file.
pub trait Serialize: TypeHash + ReprHash {
    /// Serialize the type using the given backend.
    fn serialize(&self, backend: &mut impl WriteNoStd) -> Result<usize> {
        let mut write_with_pos = WriterWithPos::new(backend);
        self.serialize_on_field_write(&mut write_with_pos)?;
        Ok(write_with_pos.pos())
    }

    /// Serialize the type using the given backend and return a [schema](Schema)
    /// describing the data that has been written.
    ///
    /// This method is mainly useful for debugging and to check cross-language
    /// interoperability.
    fn serialize_with_schema(&self, backend: &mut impl WriteNoStd) -> Result<Schema> {
        let mut writer_with_pos = WriterWithPos::new(backend);
        let mut schema_writer = SchemaWriter::new(&mut writer_with_pos);
        self.serialize_on_field_write(&mut schema_writer)?;
        Ok(schema_writer.schema)
    }

    /// Serialize the type using the given [`WriteWithNames`].
    fn serialize_on_field_write(&self, backend: &mut impl WriteWithNames) -> Result<()>;

    /// Convenience method to serialize to a file.
    fn store(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = std::fs::File::create(path).map_err(Error::FileOpenError)?;
        let mut buf_writer = BufWriter::new(file);
        self.serialize(&mut buf_writer)?;
        Ok(())
    }
}

/// Inner trait to implement serialization of a type. This trait exists
/// to separate the user-facing [`Serialize`] trait from the low-level
/// serialization mechanism of [`SerializeInner::_serialize_inner`]. Moreover,
/// it makes it possible to behave slighly differently at the top
/// of the recursion tree (e.g., to write the endianness marker), and to prevent
/// the user from modifying the methods in [`Serialize`].
///
/// The user should not implement this trait directly, but rather derive it.
pub trait SerializeInner {
    /// Inner constant used by the derive macros to keep
    /// track recursively of whether the type
    /// satisfies the conditions for being zero-copy. It is checked
    /// at runtime against the trait implemented by the type, and
    /// if a [`ZeroCopy`] type has this constant set to `false`
    /// serialization will panic.
    const IS_ZERO_COPY: bool;

    /// Inner constant used by the derive macros to keep
    /// track of whether all fields of a type are zero-copy
    /// but neither the attribute `#[zero_copy]` nor the attribute `#[deep_copy]`
    /// was specified. It is checked at runtime, and if it is true
    /// a warning will be issued, as the type could be zero-copy,
    /// which would be more efficient.
    const ZERO_COPY_MISMATCH: bool;

    /// Serialize this structure using the given backend.
    fn _serialize_inner(&self, backend: &mut impl WriteWithNames) -> Result<()>;
}

/// Blanket implementation that prevents the user from overwriting the
/// methods in [`Serialize`].
///
/// This implementation [writes a header](`write_header`) containing some hashes
/// and debug information and then delegates to [WriteWithNames::write].
impl<T: SerializeInner + TypeHash + ReprHash> Serialize for T {
    /// Serialize the type using the given [`WriteWithNames`].
    fn serialize_on_field_write(&self, backend: &mut impl WriteWithNames) -> Result<()> {
        write_header::<Self>(backend)?;
        backend.write("ROOT", self)?;
        backend.flush()
    }
}

/// Write the header.
///
/// Must be kept in sync with [`crate::deser::check_header`].
pub fn write_header<T: TypeHash + ReprHash>(backend: &mut impl WriteWithNames) -> Result<()> {
    backend.write("MAGIC", &MAGIC)?;
    backend.write("VERSION_MAJOR", &VERSION.0)?;
    backend.write("VERSION_MINOR", &VERSION.1)?;
    backend.write("USIZE_SIZE", &(core::mem::size_of::<usize>() as u8))?;

    let mut type_hasher = xxhash_rust::xxh3::Xxh3::new();
    T::type_hash(&mut type_hasher);

    let mut repr_hasher = xxhash_rust::xxh3::Xxh3::new();
    let mut offset_of = 0;
    T::repr_hash(&mut repr_hasher, &mut offset_of);

    backend.write("TYPE_HASH", &type_hasher.finish())?;
    backend.write("REPR_HASH", &repr_hasher.finish())?;
    backend.write("TYPE_NAME", &core::any::type_name::<T>().to_string())
}

/// A helper trait that makes it possible to implement differently
/// serialization for [`crate::traits::ZeroCopy`] and [`crate::traits::DeepCopy`] types.
/// See [`crate::traits::CopyType`] for more information.
pub trait SerializeHelper<T: CopySelector> {
    fn _serialize_inner(&self, backend: &mut impl WriteWithNames) -> Result<()>;
}

#[derive(Debug)]
/// Errors that can happen during serialization.
pub enum Error {
    /// The underlying writer returned an error.
    WriteError,
    /// [`Serialize::store`] could not open the provided file.
    FileOpenError(std::io::Error),
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::WriteError => write!(f, "Write error during ε-serde serialization"),
            Self::FileOpenError(error) => {
                write!(
                    f,
                    "Error opening file during ε-serde serialization: {}",
                    error
                )
            }
        }
    }
}
