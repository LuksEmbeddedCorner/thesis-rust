// Re-export the prelude of embedded-hal
pub use embedded_hal::prelude::*;

// Bring the extendions into scope
pub use crate::gpio::gpio_extension::GpioExtension as _stm32f217_hal_gpio_GpioExtension;
pub use crate::rcc::RccExtension as _stm32f217_hal_rcc_RccExtension;
pub use crate::time::U32Ext as _stm32f217_hal_time_U32Extension;
