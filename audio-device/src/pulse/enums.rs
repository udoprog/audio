use pulse_sys as pulse;
use std::fmt;

macro_rules! decl_enum {
    (
        $(#[doc = $doc:literal])*
        #[repr($ty:ident)]
        $vis:vis enum $name:ident {
            $(
                $(#[$m:meta])*
                $a:ident = $b:ident
            ),* $(,)?
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        #[repr($ty)]
        $vis enum $name {
            $(
                $(#[$m])*
                #[allow(missing_docs)]
                $a = pulse::$b,
            )*
        }

        impl $name {
            /// Parse the given enum from a value.
            $vis fn from_value(value: $ty) -> Option<Self> {
                Some(match value {
                    $(pulse::$b => Self::$a,)*
                    _ => return None,
                })
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let id = match self {
                    $(Self::$a => stringify!($a),)*
                };

                f.write_str(id)
            }
        }
    }
}

decl_enum! {
    /// The state of a connection context.
    #[repr(u32)]
    pub enum ContextState {
        /// The context hasn't been connected yet.
        Unconnected = PA_CONTEXT_UNCONNECTED,
        /// A connection is being established.
        Connecting = PA_CONTEXT_CONNECTING,
        /// The client is authorizing itself to the daemon.
        Authorizing = PA_CONTEXT_AUTHORIZING,
        /// The client is passing its application name to the daemon.
        SettingName = PA_CONTEXT_SETTING_NAME,
        /// The connection is established, the context is ready to execute operations.
        Ready = PA_CONTEXT_READY,
        /// The connection failed or was disconnected.
        Failed = PA_CONTEXT_FAILED,
        /// The connection was terminated cleanly.
        Terminated = PA_CONTEXT_TERMINATED,
    }
}
