use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::BufStream;
use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Close, Connect, Connection, Runtime};

use crate::protocol::Capabilities;
use crate::{MySql, MySqlConnectOptions};

mod close;
mod connect;
mod ping;
mod stream;

/// A single connection (also known as a session) to a MySQL database server.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnection<Rt>
where
    Rt: Runtime,
{
    stream: BufStream<Rt, NetStream<Rt>>,
    connection_id: u32,

    // the capability flags are used by the client and server to indicate which
    // features they support and want to use.
    capabilities: Capabilities,

    // the sequence-id is incremented with each packet and may wrap around. It starts at 0 and is
    // reset to 0 when a new command begins in the Command Phase.
    sequence_id: u8,
}

impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
{
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self {
            stream: BufStream::with_capacity(stream, 4096, 1024),
            connection_id: 0,
            sequence_id: 0,
            capabilities: Capabilities::PROTOCOL_41 | Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::IGNORE_SPACE
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                // | Capabilities::MULTI_STATEMENTS
                // | Capabilities::MULTI_RESULTS
                // | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                // | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                // | Capabilities::SESSION_TRACK
                | Capabilities::DEPRECATE_EOF,
        }
    }
}

impl<Rt> Debug for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    type Database = MySql;

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.ping_async())
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    fn connect(url: &str) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<Self>>
    where
        Self: Sized,
        Rt: sqlx_core::Async,
    {
        use sqlx_core::ConnectOptions;

        let options = url.parse::<Self::Options>();
        Box::pin(async move { options?.connect().await })
    }
}

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.close_async())
    }
}

#[cfg(feature = "blocking")]
mod blocking {
    use super::{MySqlConnectOptions, MySqlConnection};

    use sqlx_core::blocking::{Close, Connect, Connection, Runtime};

    impl<Rt: Runtime> Connection<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn ping(&mut self) -> sqlx_core::Result<()> {
            self.ping()
        }
    }

    impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn connect(url: &str) -> sqlx_core::Result<Self>
        where
            Self: Sized,
        {
            Self::connect(&url.parse::<MySqlConnectOptions<Rt>>()?)
        }
    }

    impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn close(self) -> sqlx_core::Result<()> {
            self.close()
        }
    }
}
