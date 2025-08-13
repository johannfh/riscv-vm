//! # RISC-V Constants
//!
//! This module defines constants for RISC-V, to help with stuff like calling convention
//! implementation. See also: [https://riscv.org/wp-content/uploads/2024/12/riscv-calling.pdf]
//!
//! ## Argument Registers a0-a7
//!
//! `a0` to `a7` are the argument registers used in the RISC-V calling convention RVG.

macro_rules! register {
    ($name:ident, $value:expr) => {
        ::paste::paste! {
            #[doc = "Register [`" $name "`] has the index: `" $value "`."]
            #[allow(non_upper_case_globals)]
            pub const $name: usize = $value;
        }
    };
}

macro_rules! register_block {
    (
        $(($name:ident, $value:expr)),*
    ) => {
        $( register!($name, $value); )*
    };
}

macro_rules! register_alias {
    (
        $(#[$attr:meta])*
        $name:ident, $alias:ident
    ) => {
        ::paste::paste! {
            $(#[$attr])*
            #[doc = "Alias for register: [`" $name "`] -> [`" $alias "`]"]
            #[allow(non_upper_case_globals)]
            pub const $name: usize = $alias;
        }
    };
}

macro_rules! register_alias_block {
    (
        $(#[$attr:meta])*
        $(($name:ident, $alias:ident)),*
    ) => {
        __register_alias_recursive! {
            ( $(#[$attr])* ) // the attributes for the aliases
            ( $( ($name, $alias) ),* )
        }
    };
}

macro_rules! __register_alias_recursive {
    (
        ( $(#[$attr:meta])* ) // the attributes for the aliases
        () // the emtpy list of aliases
    ) => {};

    (
        ( $(#[$attr:meta])* ) // the attributes for the aliases
        ( ($name:ident, $alias:ident) $(, ($rest_name:ident, $rest_alias:ident) )* )
    ) => {
        register_alias! {
            $(#[$attr])*
            $name, $alias
        }
        __register_alias_recursive! {
            ( $(#[$attr])* ) // the attributes for the aliases
            ( $( ($rest_name, $rest_alias) ),* ) // the rest of the aliases to iterate over
        }
    };
}

// RISC-V registers
register_block! {
    (x0, 0), (x1, 1), (x2, 2), (x3, 3),
    (x4, 4), (x5, 5), (x6, 6), (x7, 7),
    (x8, 8), (x9, 9), (x10, 10), (x11, 11),
    (x12, 12), (x13, 13), (x14, 14), (x15, 15),
    (x16, 16), (x17, 17), (x18, 18), (x19, 19),
    (x20, 20), (x21, 21), (x22, 22), (x23, 23),
    (x24, 24), (x25, 25), (x26, 26), (x27, 27),
    (x28, 28), (x29, 29), (x30, 30), (x31, 31)
}

register_alias! {
    /// Zero register. Always holds the value zero.
    /// This register cannot be modified, but be used to discard values.
    /// This is done by setting [`zero`] as the destination register in an instruction.
    zero, x0
}

register_alias! {
    /// Return address, Saved by caller
    ra, x1
}

register_alias! {
    /// Stack pointer, Saved by callee.
    sp, x2
}

register_alias! {
    /// Global pointer.
    gp, x3
}

register_alias! {
    /// Thread pointer.
    tp, x4
}

register_alias! {
    /// Frame pointer, Saved by callee.
    fp, x8
}

// --- Argument registers a0-a7 -> x10-x17 ---
register_alias_block! {
    /// Argument register, saved by caller.
    (a0, x10), (a1, x11), (a2, x12), (a3, x13),
    (a4, x14), (a5, x15), (a6, x16), (a7, x17)
}

// --- Saved registers ---
// s0-s1 -> x8-x9
// s2-s11 -> x18-x27
register_alias_block! {
    /// Saved register, saved by callee.
    (s0, x8), (s1, x9), (s2, x18), (s3, x19),
    (s4, x20), (s5, x21), (s6, x22), (s7, x23),
    (s8, x24), (s9, x25), (s10, x26), (s11, x27)
}

// --- Temporary registers ---
// t0-t2 -> x5-x7
// t3-t6 -> x28-x31
register_alias_block! {
    /// Temporary register, saved by caller.
    (t0, x5), (t1, x6), (t2, x7), (t3, x28),
    (t4, x29), (t5, x30), (t6, x31)
}

