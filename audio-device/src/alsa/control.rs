use crate::alsa::{ControlElementInterface, Result};
use crate::libc as c;
use alsa_sys as alsa;
use std::ffi::CStr;
use std::mem;
use std::ptr;

/// A control associated with a device.
///
/// See [Control::open].
pub struct Control {
    tag: ste::Tag,
    pub(super) handle: ptr::NonNull<alsa::snd_ctl_t>,
}

impl Control {
    /// Opens a CTL.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let pcm = alsa::Control::open(&name)?;
    /// # Ok(()) }
    /// ```
    pub fn open(name: &CStr) -> Result<Self> {
        unsafe {
            let mut handle = mem::MaybeUninit::uninit();

            errno!(alsa::snd_ctl_open(handle.as_mut_ptr(), name.as_ptr(), 0,))?;

            Ok(Self {
                tag: ste::Tag::current_thread(),
                handle: ptr::NonNull::new_unchecked(handle.assume_init()),
            })
        }
    }

    /// get identifier of CTL handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let pcm = alsa::Control::open(&name)?;
    /// println!("control: {}", pcm.name().to_str()?);
    /// # Ok(()) }
    /// ```
    pub fn name(&self) -> &CStr {
        self.tag.ensure_on_thread();

        unsafe { CStr::from_ptr(alsa::snd_ctl_name(self.handle.as_ptr())) }
    }

    /// Get a list of element identifiers.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// if let Some(element) = element_list.get(0) {
    ///     println!("name: {}", element.name().to_str()?);
    ///     println!("interface: {}", element.interface());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn element_list(&self) -> Result<ControlElementList> {
        self.tag.ensure_on_thread();

        unsafe {
            let mut handle = mem::MaybeUninit::uninit();
            errno!(alsa::snd_ctl_elem_list_malloc(handle.as_mut_ptr()))?;
            let handle = ptr::NonNull::new_unchecked(handle.assume_init());

            let mut list = ControlElementList {
                handle,
                space: false,
                count: 0,
                used: 0,
            };

            errno!(alsa::snd_ctl_elem_list(
                self.handle.as_ptr(),
                list.handle.as_mut()
            ))?;
            let count = alsa::snd_ctl_elem_list_get_count(list.handle.as_mut());
            list.count = count;

            errno!(alsa::snd_ctl_elem_list_alloc_space(
                list.handle.as_mut(),
                count
            ))?;
            list.space = true;
            errno!(alsa::snd_ctl_elem_list(
                self.handle.as_ptr(),
                list.handle.as_mut()
            ))?;
            list.used = alsa::snd_ctl_elem_list_get_used(list.handle.as_ref());
            Ok(list)
        }
    }
}

// Safety: [Control] is tagged with the thread its created it and is ensured not to
// leave it.
unsafe impl Send for Control {}

impl Drop for Control {
    fn drop(&mut self) {
        unsafe { alsa::snd_ctl_close(self.handle.as_ptr()) };
    }
}

/// Reference to a control element from a [ControlElementList].
///
/// Fetched with [ControlElementList::get].
pub struct ControlElement<'a> {
    handle: &'a ptr::NonNull<alsa::snd_ctl_elem_list_t>,
    index: c::c_uint,
}

impl ControlElement<'_> {
    /// Get interface part of CTL element identifier for an entry of a CTL
    /// element identifiers list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// if let Some(element) = element_list.get(0) {
    ///     println!("interface: {}", element.interface());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn interface(&self) -> ControlElementInterface {
        unsafe {
            let interface = alsa::snd_ctl_elem_list_get_interface(self.handle.as_ref(), self.index);
            ControlElementInterface::from_value(interface).expect("bad control element interface")
        }
    }

    /// get identifier of CTL handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// if let Some(element) = element_list.get(0) {
    ///     println!("name: {}", element.name().to_str()?);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(alsa::snd_ctl_elem_list_get_name(
                self.handle.as_ref(),
                self.index,
            ))
        }
    }

    /// get identifier of CTL handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// if let Some(element) = element_list.get(0) {
    ///     println!("index: {}", element.index());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn index(&self) -> c::c_uint {
        unsafe { alsa::snd_ctl_elem_list_get_index(self.handle.as_ref(), self.index) }
    }
}

/// A list of control elements.
pub struct ControlElementList {
    handle: ptr::NonNull<alsa::snd_ctl_elem_list_t>,
    space: bool,
    count: c::c_uint,
    used: c::c_uint,
}

impl ControlElementList {
    /// Get number of used entries in CTL element identifiers list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    /// dbg!(element_list.used());
    /// # Ok(()) }
    /// ```
    pub fn used(&self) -> c::c_uint {
        unsafe { alsa::snd_ctl_elem_list_get_used(self.handle.as_ref()) }
    }

    /// Get total count of elements present in CTL device.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    /// dbg!(element_list.count());
    /// # Ok(()) }
    /// ```
    pub fn count(&self) -> c::c_uint {
        self.count
    }

    /// Get the control element at the given index.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// if let Some(element) = element_list.get(0) {
    ///     println!("{}", element.name().to_str()?);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn get(&self, index: c::c_uint) -> Option<ControlElement<'_>> {
        if index >= self.used {
            return None;
        }

        Some(ControlElement {
            handle: &self.handle,
            index,
        })
    }

    /// Construct an iterator over the list of control elements.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CString::new("hw:0")?;
    /// let control = alsa::Control::open(&name)?;
    /// let element_list = control.element_list()?;
    ///
    /// for element in element_list.iter() {
    ///     println!("{}", element.name().to_str()?);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn iter(&self) -> ControlElementListIter<'_> {
        ControlElementListIter {
            handle: &self.handle,
            index: 0,
            used: self.used,
        }
    }
}

impl Drop for ControlElementList {
    fn drop(&mut self) {
        unsafe {
            if self.space {
                let _ = alsa::snd_ctl_elem_list_free_space(self.handle.as_ptr());
            }

            let _ = alsa::snd_ctl_elem_list_free(self.handle.as_ptr());
        };
    }
}

/// An iterator over available control elements.
///
/// See [ControlElementList::iter].
pub struct ControlElementListIter<'a> {
    handle: &'a ptr::NonNull<alsa::snd_ctl_elem_list_t>,
    index: c::c_uint,
    used: c::c_uint,
}

impl<'a> Iterator for ControlElementListIter<'a> {
    type Item = ControlElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.used {
            return None;
        }

        let index = self.index;
        self.index += 1;

        Some(ControlElement {
            handle: self.handle,
            index,
        })
    }
}
