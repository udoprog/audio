use crate::alsa::{CString, Result};
use alsa_sys as alsa;
use libc::c_int;
use std::ffi::CStr;
use std::mem;

/// Construct an iterator over sounds cards.
///
/// # Examples
///
/// ```rust,no_run
/// use audio_device::alsa;
///
/// # fn main() -> anyhow::Result<()> {
/// for card in alsa::cards() {
///     let card = card?;
///     println!("{}", card.name()?.to_str()?);
/// }
/// # Ok(()) }
/// ```
pub fn cards() -> Cards {
    Cards { index: -1 }
}

/// An iterator over available cards.
///
/// See [cards].
pub struct Cards {
    index: c_int,
}

impl Iterator for Cards {
    type Item = Result<Card>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            if let Err(e) = errno!(unsafe { alsa::snd_card_next(&mut self.index) }) {
                Err(e)
            } else {
                if self.index == -1 {
                    return None;
                }

                Ok(Card { index: self.index })
            },
        )
    }
}

/// A reference to a card.
pub struct Card {
    index: c_int,
}

impl Card {
    /// Open the given pcm device identified by name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CStr::from_bytes_with_nul(b"hw:0\0")?;
    ///
    /// let pcm = alsa::Card::open(name)?;
    /// # Ok(()) }
    /// ```
    pub fn open(name: &CStr) -> Result<Self> {
        unsafe {
            let index = errno!(alsa::snd_card_get_index(
                name.to_bytes().as_ptr() as *const i8
            ))?;
            Ok(Self { index })
        }
    }

    /// Get the index of the card.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// for card in alsa::cards() {
    ///     let card = card?;
    ///     println!("{}", card.index());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn index(&self) -> c_int {
        self.index
    }

    /// Get the name of the card.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// for card in alsa::cards() {
    ///     let card = card?;
    ///     println!("{}", card.name()?.to_str()?);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn name(&self) -> Result<CString> {
        unsafe {
            let mut ptr = mem::MaybeUninit::uninit();

            if let Err(e) = errno!(alsa::snd_card_get_name(self.index, ptr.as_mut_ptr())) {
                return Err(e);
            }

            let ptr = ptr.assume_init();
            Ok(CString::from_raw(ptr))
        }
    }

    /// Get the long name of the card.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// for card in alsa::cards() {
    ///     let card = card?;
    ///     println!("{}", card.long_name()?.to_str()?);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn long_name(&self) -> Result<CString> {
        unsafe {
            let mut ptr = mem::MaybeUninit::uninit();

            if let Err(e) = errno!(alsa::snd_card_get_longname(self.index, ptr.as_mut_ptr())) {
                return Err(e);
            }

            let ptr = ptr.assume_init();
            Ok(CString::from_raw(ptr))
        }
    }
}
