/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// Example of zero-copy deserialization of a zero-copy struct.
use epserde::prelude::*;

#[derive(Epserde, Copy, Clone, Debug)]
#[repr(C)]
#[zero_copy]
struct Data {
    a: [usize; 100],
}

fn main() {
    let a = Data { a: [1_usize; 100] };
    let mut buf = epserde::new_aligned_cursor();
    // Serialize
    let _bytes_written = a.serialize(&mut buf).unwrap();

    // Do a full-copy deserialization
    buf.set_position(0);
    let full = Data::deserialize_full(&mut buf).unwrap();
    println!(
        "Full-copy deserialization type: {}",
        std::any::type_name::<Data>(),
    );
    println!("Value: {:x?}", full);

    println!("\n");

    // Do an ε-copy deserialization (which will be a zero-copy deserialization)
    let buf = buf.into_inner();
    let eps = Data::deserialize_eps(&buf).unwrap();
    println!(
        "ε-copy deserialization type: {}",
        std::any::type_name::<<Data as DeserializeInner>::DeserType<'_>>(),
    );
    println!("Value: {:x?}", eps);
}
