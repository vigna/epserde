/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Traits computing structural information about a type.

*/

/// Compute a type hash and a representational hash for a type.
/// They are used during deserialization to
/// check that the type of the data matches the type of the value being
/// deserialized into.
pub trait TypeHash {
    fn type_hash(
        type_hasher: &mut impl core::hash::Hasher,
        repr_hasher: &mut impl core::hash::Hasher,
        offset_of: &mut usize,
    );

    /// Call [`TypeHash::type_hash`] on a value.
    fn type_hash_val(
        &self,
        type_hasher: &mut impl core::hash::Hasher,
        repr_hasher: &mut impl core::hash::Hasher,
        offset_of: &mut usize,
    ) {
        Self::type_hash(type_hasher, repr_hasher, offset_of);
    }
}

/// A trait giving the maximum size of a primitive field in a type.
///
/// We use the value returned by [`MaxSizeOf::max_size_of`]
/// to generate padding before storing a zero-copy type. Note that this
/// is different from the padding used to align the same type inside
/// a struct, which is not under our control and is
/// given by [`core::mem::align_of`].
///
/// In this way we increase interoperability between architectures
/// with different alignment requirements for the same types (e.g.,
/// 4 or 8 bytes for `u64`).
pub trait MaxSizeOf: Sized {
    fn max_size_of() -> usize;
}
