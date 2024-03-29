macro_rules! construct_zeromq {
    (
        name = $name:ident,
        socket_type = $socket_type:expr,
        category = $category:path
    ) => {
        #[doc = concat!("The [`", stringify!($category), "`] of ZeroMQ.")]
        pub struct $name {
            socket: zmq2::Socket,
        }

        impl $name {
            pub fn new() -> $crate::message::MessageResult<Self> {
                let context = zmq2::Context::new();
                let socket = context
                    .socket($socket_type)
                    .map_err($crate::message::zeromq::ZeromqError::CreateSocketFailed)?;

                Ok(Self { socket })
            }
        }

        impl AsRef<zmq2::Socket> for $name {
            fn as_ref(&self) -> &zmq2::Socket {
                &self.socket
            }
        }

        impl AsMut<zmq2::Socket> for $name {
            fn as_mut(&mut self) -> &mut zmq2::Socket {
                &mut self.socket
            }
        }

        impl From<$name> for zmq2::Socket {
            fn from(s: $name) -> Self {
                s.socket
            }
        }

        impl $category for $name {}
    };
}

pub(super) use construct_zeromq;
