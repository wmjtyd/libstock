macro_rules! construct_nanomsg {
    (
        name = $name:ident,
        socket_type = $socket_type:expr,
        category = $category:path
    ) => {
        pub struct $name {
            socket: nanomsg::Socket,
            endpoint: std::collections::HashMap<String, nanomsg::Endpoint>,
        }

        impl $name {
            pub fn new() -> $crate::message::MessageResult<Self> {
                Ok(Self {
                    socket: nanomsg::Socket::new($socket_type)
                        .map_err($crate::message::nanomsg::NanomsgError::CreateSocketFailed)?,
                    endpoint: Default::default(),
                })
            }
        }

        impl std::ops::Deref for $name {
            type Target = nanomsg::Socket;

            fn deref(&self) -> &Self::Target {
                &self.socket
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.socket
            }
        }

        impl From<$name> for nanomsg::Socket {
            fn from(s: $name) -> Self {
                s.socket
            }
        }

        impl $category for $name {}
    };
}

pub(super) use construct_nanomsg;
