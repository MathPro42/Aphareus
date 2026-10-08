mod castling;
mod color;
mod piece;
mod square;

pub use castling::{CastleSide, CastlingRights};
pub use color::Color;
pub use piece::{Piece, PieceType};
pub use square::{File, Rank, Square};

/// Defines `from_index_bounded` for each listed enum: its one conversion
/// from an integer, and the only `unsafe` block of these types.
macro_rules! impl_from_index_bounded {
    ($($ty:ident),+) => {$(
        impl $ty {
            #[inline(always)]
            pub(crate) const fn from_index_bounded(index: u32) -> $ty {
                debug_assert!(
                    (index as usize) < $ty::COUNT,
                    concat!(stringify!($ty), " index out of range")
                );
                let last = ($ty::COUNT - 1) as u32;
                // `COUNT` is a constant, so only one branch is compiled.
                let index = if $ty::COUNT.is_power_of_two() {
                    index & last
                } else if index > last {
                    last
                } else {
                    index
                };
                // SAFETY: `index` is clamped to `0..COUNT`, and the const
                // block below checks that `$ty` is one byte with discriminants
                // `0..COUNT`, so `index` is a valid `$ty`.
                unsafe { core::mem::transmute::<u8, $ty>(index as u8) }
            }
        }

        // The layout `from_index_bounded` relies on: one byte, and `ALL[i]`
        // has the discriminant `i` for every index.
        const _: () = {
            assert!(core::mem::size_of::<$ty>() == 1);
            let mut i = 0;
            while i < $ty::COUNT {
                assert!($ty::ALL[i] as usize == i);
                i += 1;
            }
        };
    )+};
}

impl_from_index_bounded!(Square, File, Rank, Color, PieceType, Piece);
