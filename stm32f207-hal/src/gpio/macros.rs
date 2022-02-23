macro_rules! gpio_read_bits {
    ($port:ident, $register_name:ident) => {
        gpio_read_bits_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I']
            [A B C D E F G H I]
            $port, $register_name
        )
    };
}
pub(crate) use gpio_read_bits;

macro_rules! gpio_read_bits_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident) => {
        paste::item! {
            // This match statement is guaranteed to be optimised
            // If Port is known at compile time
            match PORT {
                $($character => {
                    let gpio = &*::stm32f2::stm32f217::[< GPIO $letter >]::ptr();
                    gpio.$register_name.read().bits()
                })+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}
pub(crate) use gpio_read_bits_inner;

macro_rules! gpio_write {
    ($port:ident, $register_name:ident, |$write:ident| $expr:expr) => {
        gpio_write_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I']
            [A B C D E F G H I]
            $port, $register_name, |$write| $expr
        );
    };
}
pub(crate) use gpio_write;

macro_rules! gpio_write_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident, |$write:ident| $expr:expr) => {
        paste::item! {
            // This match statement is guaranteed to be optimised
            // If Port is known at compile time
            match PORT {
                $($character => {
                    let gpio = &*::stm32f2::stm32f217::[< GPIO $letter >]::ptr();
                    gpio.$register_name.write(|$write| $expr);
                })+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}
pub(crate) use gpio_write_inner;

// Adapted from: https://www.ecorax.net/macro-bunker-2/
macro_rules! gpio_modify {
    ($port:ident, $register_name:ident, |$read:ident, $write:ident| $block:block) => {
        gpio_modify_inner!(
            ['A' 'B' 'C' 'D' 'E' 'F' 'G' 'H' 'I']
            [A B C D E F G H I]
            $port, $register_name, |$read, $write| $block
        );
    };
}
pub(crate) use gpio_modify;

// Adapted from: https://www.ecorax.net/macro-bunker-2/
macro_rules! gpio_modify_inner {
    ([$($character:literal)+] [$($letter:ident)+] $port:ident, $register_name:ident, |$read:ident, $write:ident| $block:block) => {
        paste::item! {
            // This match statement is guaranteed to be optimised
            // If Port is known at compile time
            match PORT {
                $($character => {
                    let gpio = &*::stm32f2::stm32f217::[< GPIO $letter >]::ptr();
                    gpio.$register_name.modify(|$read, $write| $block);
                })+
                _ => core::panic!("Unexpected port"),
            }
        }
    };
}
pub(crate) use gpio_modify_inner;
