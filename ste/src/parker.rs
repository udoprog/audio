#[cfg(feature = "parking-lot-parker")]
mod parking_lot;
#[cfg(feature = "parking-lot-parker")]
pub(crate) use self::parking_lot::{new, Unparker};

#[cfg(not(feature = "parking-lot-parker"))]
mod thread;
#[cfg(not(feature = "parking-lot-parker"))]
pub(crate) use self::thread::{new, Unparker};
