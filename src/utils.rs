#[macro_export]
macro_rules! format_u32_le_bits {
    ($val:expr) => {
        format!(
            "{:08b}_{:08b}_{:08b}_{:08b}",
            ($val >> 24) as u8,
            ($val >> 16) as u8,
            ($val >> 8) as u8,
            $val as u8
        )
    };
}

macro_rules! impl_sign_extend {
    ($source_type:ty, $target_type:ty) => {
        ::paste::paste! {
            pub fn [<sign_extend_ $source_type _to_ $target_type>](value: $source_type, size: usize) -> $target_type {
                const SOURCE_SIZE: usize = ::std::mem::size_of::<$source_type>() * 8;
                ::static_assertions::assert_eq_size!($source_type, $target_type);
                assert!(size > 0 && size <= SOURCE_SIZE, "Size must be between 1 and 32 bits");
                (value << (SOURCE_SIZE - size)) as $target_type >> (SOURCE_SIZE - size)
            }
        }
    };
}

impl_sign_extend!(u8, i8);
impl_sign_extend!(u16, i16);
impl_sign_extend!(u32, i32);
impl_sign_extend!(u64, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test cases for sign extension of all ones (resulting in -1)
    fn test_sign_extend_all_ones() {
        assert_eq!(sign_extend_u8_to_i8(0xff, 8), -1);
        assert_eq!(sign_extend_u16_to_i16(0xffff, 16), -1);
        assert_eq!(sign_extend_u32_to_i32(0xffff_ffff, 32), -1);
        assert_eq!(sign_extend_u64_to_i64(0xffff_ffff_ffff_ffff, 64), -1);
    }

    #[test]
    /// Test cases for sign extension of all zeros (resulting in 0)
    fn test_sign_extend_all_zeros() {
        assert_eq!(sign_extend_u8_to_i8(0x00, 8), 0);
        assert_eq!(sign_extend_u16_to_i16(0x0000, 16), 0);
        assert_eq!(sign_extend_u32_to_i32(0x000_0000, 32), 0);
        assert_eq!(sign_extend_u64_to_i64(0x0000_0000_0000_0000, 64), 0);
    }

    #[test]
    /// Test cases for sign extension of positive values (resulting in the same value)
    fn test_sign_extend_positive_values() {
        assert_eq!(sign_extend_u8_to_i8(0x7f, 8), 127);
        assert_eq!(sign_extend_u16_to_i16(0x7fff, 16), 32767);
        assert_eq!(sign_extend_u32_to_i32(0x7fff_ffff, 32), 2147483647);
        assert_eq!(
            sign_extend_u64_to_i64(0x7fff_ffff_ffff_ffff, 64),
            9223372036854775807
        );
    }

    #[test]
    /// Test cases for sign extension of negative values (resulting in the correct negative value)
    fn test_sign_extend_negative_values() {
        assert_eq!(sign_extend_u8_to_i8(0x80, 8), -128);
        assert_eq!(sign_extend_u16_to_i16(0x8000, 16), -32768);
        assert_eq!(sign_extend_u32_to_i32(0x8000_0000, 32), -2147483648);
        assert_eq!(
            sign_extend_u64_to_i64(0x8000_0000_0000_0000, 64),
            -9223372036854775808
        );
    }
}

