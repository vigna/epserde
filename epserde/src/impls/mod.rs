/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!

Implementations of [`SerializeInner`](crate::ser::SerializeInner)
and [`DeserializeInner`](crate::deser::DeserializeInner) for standard Rust types.

*/

pub mod array;
pub mod boxed_slice;
pub mod prim;
pub mod slice;
pub mod string;
pub mod tuple;
pub mod vec;
