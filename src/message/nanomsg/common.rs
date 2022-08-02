macro_rules! construct_nanomsg {
    (
        name = $name:ident,
        socket_type = $socket_type:expr,
        category = $category:path
    ) => {
        #[doc = concat!("The [`", stringify!($category), "`] of Nanomsg.")]
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

        impl AsRef<nanomsg::Socket> for $name {
            fn as_ref(&self) -> &nanomsg::Socket {
                &self.socket
            }
        }

        impl AsMut<nanomsg::Socket> for $name {
            fn as_mut(&mut self) -> &mut nanomsg::Socket {
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
