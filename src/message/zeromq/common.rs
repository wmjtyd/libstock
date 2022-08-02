macro_rules! construct_zeromq {
    (
        name = $name:ident,
        socket_type = $socket_type:expr,
        category = $category:path
    ) => {
        pub struct $name {
            socket: zmq::Socket,
        }

        impl $name {
            pub fn new() -> $crate::message::MessageResult<Self> {
                let context = zmq::Context::new();
                let socket = context
                    .socket($socket_type)
                    .map_err($crate::message::zeromq::ZeromqError::CreateSocketFailed)?;

                Ok(Self { socket })
            }
        }

        impl std::ops::Deref for $name {
            type Target = zmq::Socket;

            fn deref(&self) -> &Self::Target {
                &self.socket
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.socket
            }
        }

        impl From<$name> for zmq::Socket {
            fn from(s: $name) -> Self {
                s.socket
            }
        }

        impl $category for $name {}
    };
}

pub(super) use construct_zeromq;
