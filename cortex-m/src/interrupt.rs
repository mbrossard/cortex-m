//! Interrupts

pub use bare_metal::{CriticalSection, Mutex, Nr};

/// Trait for enums of external interrupt numbers.
///
/// This trait should be implemented by a peripheral access crate (PAC)
/// on its enum of available external interrupts for a specific device.
/// Each variant must convert to a u16 of its interrupt number,
/// which is its exception number - 16.
///
/// # Safety
///
/// This trait must only be implemented on enums of device interrupts. Each
/// enum variant must represent a distinct value (no duplicates are permitted),
/// and must always return the same value (do not change at runtime).
///
/// These requirements ensure safe nesting of critical sections.
pub unsafe trait InterruptNumber: Copy {
    /// Return the interrupt number associated with this variant.
    ///
    /// See trait documentation for safety requirements.
    fn number(self) -> u16;
}

/// Implement InterruptNumber for the old bare_metal::Nr trait.
/// This implementation is for backwards compatibility only and will be removed in cortex-m 0.8.
unsafe impl<T: Nr + Copy> InterruptNumber for T {
    #[inline]
    fn number(self) -> u16 {
        self.nr() as u16
    }
}

/// Disables all interrupts in the current core.
#[inline]
pub fn disable() {
    call_asm!(__cpsid());
}

/// Enables all the interrupts in the current core.
///
/// # Safety
///
/// - Do not call this function inside a critical section.
#[inline]
pub unsafe fn enable() {
    call_asm!(__cpsie());
}

/// Execute closure `f` with interrupts disabled in the current core.
///
/// This method does not synchronize multiple cores and may disable required
/// interrupts on some platforms; see the `critical-section` crate for a cross-platform
/// way to enter a critical section which provides a `CriticalSection` token.
///
/// This crate provides an implementation for `critical-section` suitable for single-core systems,
/// based on disabling all interrupts. It can be enabled with the `critical-section-single-core` feature.
#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let primask = crate::register::primask::read();

    // disable interrupts
    disable();

    let r = f();

    // If the interrupts were active before our `disable` call, then re-enable
    // them. Otherwise, keep them disabled
    if primask.is_active() {
        unsafe { enable() }
    }

    r
}
