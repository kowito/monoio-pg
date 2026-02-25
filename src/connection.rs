use crate::auth;
use crate::client::{Column, Row};
use crate::codec::PostgresCodec;
use crate::error::{Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use bytes::BytesMut;
use fallible_iterator::FallibleIterator;
use monoio::io::sink::Sink;
use monoio::io::stream::Stream;
use monoio::net::TcpStream;
use monoio_codec::Framed;
use postgres_protocol::message::backend;
use postgres_protocol::message::frontend;
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub struct Connection {
    framed: Framed<TcpStream, PostgresCodec>,
}

impl Connection {
    pub async fn connect(
        addr: &str,
        user: &str,
        password: Option<&str>,
        database: Option<&str>,
    ) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let mut framed = Framed::new(stream, PostgresCodec);

        // 1. Send Startup Message
        let mut params = vec![("user", user)];
        if let Some(db) = database {
            params.push(("database", db));
        }
        params.push(("client_encoding", "UTF8"));

        let mut buf = BytesMut::new();
        frontend::startup_message(params.into_iter(), &mut buf)
            .map_err(|e| Error::Protocol(e.to_string()))?;

        // This is a bit tricky since Framed's encoder expects BytesMut
        // But we want to send the raw bytes.
        // For monoio-codec, we can use framed.send().
        // However, we need to handle the handshake before returning the connection.

        // Manual write for handshake might be easier if we don't want to wrap every frontend msg in a type.
        // Let's implement a simple send_raw for now or just use the codec's encode.

        // framed.send(buf).await?; // This would require Framed to be Sink-like or handle it manually.
        // Actually, monoio-codec Framed has next() and send().

        // Let's just use the inner stream for handshake for simplicity, or implement a proper AsyncWrite-based handshake.
        // But monoio is different from tokio.

        // Correct way in monoio-codec:
        framed
            .send(buf)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        framed
            .flush()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        let mut parameters = std::collections::HashMap::new();

        // 2. Handle Authentication
        let mut scram_state: Option<(auth::ScramClient, [u8; 32])> = None;
        loop {
            let msg = framed.next().await.ok_or(Error::Closed)??;
            match msg {
                backend::Message::AuthenticationOk => {}
                backend::Message::AuthenticationCleartextPassword => {
                    let pass = password.ok_or(Error::Authentication("Password required".into()))?;
                    let mut buf = BytesMut::new();
                    frontend::password_message(pass.as_bytes(), &mut buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;
                    framed
                        .send(buf)
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    framed
                        .flush()
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                }
                backend::Message::AuthenticationMd5Password(body) => {
                    let pass = password.ok_or(Error::Authentication("Password required".into()))?;
                    let encrypted = auth::md5_encrypt(user, pass, &body.salt());
                    let mut buf = BytesMut::new();
                    frontend::password_message(encrypted.as_bytes(), &mut buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;
                    framed
                        .send(buf)
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    framed
                        .flush()
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                }
                backend::Message::AuthenticationSasl(_body) => {
                    let s = auth::ScramClient::new(user, password.unwrap_or(""));
                    let mut buf = BytesMut::new();
                    frontend::sasl_initial_response(
                        "SCRAM-SHA-256",
                        s.client_first_message().as_bytes(),
                        &mut buf,
                    )
                    .map_err(|e| Error::Protocol(e.to_string()))?;
                    framed
                        .send(buf)
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    framed
                        .flush()
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    scram_state = Some((s, [0u8; 32]));
                }
                backend::Message::AuthenticationSaslContinue(body) => {
                    let (s, _) = scram_state
                        .as_mut()
                        .ok_or(Error::Authentication("SCRAM state missing".into()))?;
                    let server_first = std::str::from_utf8(body.data())
                        .map_err(|_| Error::Authentication("Invalid SCRAM utf8".into()))?;
                    let (response, server_signature) =
                        s.handle_server_first_message(server_first)?;
                    let mut buf = BytesMut::new();
                    frontend::sasl_response(response.as_bytes(), &mut buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;
                    framed
                        .send(buf)
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    framed
                        .flush()
                        .await
                        .map_err(|e| Error::Other(e.to_string()))?;
                    scram_state.as_mut().unwrap().1 = server_signature;
                }
                backend::Message::AuthenticationSaslFinal(body) => {
                    let (_, server_signature) = scram_state
                        .as_ref()
                        .ok_or(Error::Authentication("SCRAM state missing".into()))?;
                    let data = std::str::from_utf8(body.data())
                        .map_err(|_| Error::Authentication("Invalid SCRAM utf8".into()))?;
                    if !data.starts_with("v=") {
                        return Err(Error::Authentication("Missing v in SCRAM final".into()));
                    }
                    let v = BASE64
                        .decode(&data[2..])
                        .map_err(|_| Error::Authentication("Invalid v base64".into()))?;
                    if !bool::from(v.as_slice().ct_eq(server_signature)) {
                        return Err(Error::Authentication("Server signature mismatch".into()));
                    }
                }
                backend::Message::ErrorResponse(body) => {
                    let mut fields = body.fields();
                    let first = FallibleIterator::next(&mut fields)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?;
                    let msg = first
                        .map(|f: postgres_protocol::message::backend::ErrorField| {
                            String::from_utf8_lossy(f.value_bytes()).to_string()
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(Error::Authentication(msg));
                }
                backend::Message::ParameterStatus(body) => {
                    let name = body
                        .name()
                        .map_err(|e| Error::Protocol(e.to_string()))?
                        .to_string();
                    let value = body
                        .value()
                        .map_err(|e| Error::Protocol(e.to_string()))?
                        .to_string();
                    parameters.insert(name, value);
                }
                backend::Message::ReadyForQuery(_) => break,
                backend::Message::BackendKeyData(_) => {}
                _ => {}
            }
        }

        Ok(Self { framed })
    }

    pub async fn query(&mut self, query: &str) -> Result<Vec<Row>> {
        let mut buf = BytesMut::new();
        frontend::parse("", query, std::iter::empty(), &mut buf)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        // Note: formats = format codes for bound parameters. We have no parameters.
        // result_formats = format codes for results. We use 1 (binary).
        frontend::bind(
            "",
            "",
            std::iter::empty::<i16>(),
            std::iter::empty::<i32>(),
            |_: i32,
             _: &mut BytesMut|
             -> std::result::Result<
                postgres_protocol::IsNull,
                Box<dyn std::error::Error + Sync + Send>,
            > { Ok(postgres_protocol::IsNull::Yes) },
            std::iter::once(1),
            &mut buf,
        )
        .map_err(|_| Error::Protocol("Bind error".to_string()))?;
        frontend::describe(b'P', "", &mut buf).map_err(|e| Error::Protocol(e.to_string()))?;
        frontend::execute("", 0, &mut buf).map_err(|e| Error::Protocol(e.to_string()))?;
        frontend::sync(&mut buf);
        self.framed
            .send(buf)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        self.framed
            .flush()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let mut rows = Vec::new();
        let mut columns = Arc::new(Vec::new());
        let mut error = None;
        loop {
            let msg = self.framed.next().await.ok_or(Error::Closed)??;
            match msg {
                backend::Message::RowDescription(body) => {
                    let mut cols = Vec::new();
                    let mut fields = body.fields();
                    while let Some(field) = FallibleIterator::next(&mut fields)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?
                    {
                        cols.push(Column {
                            name: field.name().to_string(),
                            table_oid: field.table_oid(),
                            column_id: field.column_id(),
                            type_oid: field.type_oid(),
                            type_len: 0, // In this version, type_len might be missing or renamed
                            type_mod: field.type_modifier(),
                            format: field.format(),
                        });
                    }
                    columns = Arc::new(cols);
                }
                backend::Message::DataRow(body) => {
                    let mut data = Vec::new();
                    let mut ranges = body.ranges();
                    while let Some(range) = FallibleIterator::next(&mut ranges)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?
                    {
                        let buf = match range {
                            Some(r) => Some(bytes::Bytes::copy_from_slice(
                                &body.buffer()[r.start..r.end],
                            )),
                            None => None,
                        };
                        data.push(buf);
                    }

                    rows.push(Row {
                        columns: columns.clone(),
                        data,
                    });
                }
                backend::Message::CommandComplete(_body) => {}
                backend::Message::ReadyForQuery(_) => break,
                backend::Message::ErrorResponse(body) => {
                    let mut fields = body.fields();
                    let first = FallibleIterator::next(&mut fields)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?;
                    let msg = first
                        .map(|f: postgres_protocol::message::backend::ErrorField| {
                            String::from_utf8_lossy(f.value_bytes()).into_owned()
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    error = Some(Error::Protocol(msg));
                }
                backend::Message::ParseComplete | backend::Message::BindComplete => {}
                _ => {}
            }
        }

        if let Some(e) = error {
            return Err(e);
        }
        Ok(rows)
    }

    pub async fn execute(&mut self, query: &str) -> Result<()> {
        let mut buf = BytesMut::new();
        frontend::query(query, &mut buf).map_err(|e| Error::Protocol(e.to_string()))?;
        self.framed
            .send(buf)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        self.framed
            .flush()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let mut error = None;
        loop {
            let msg = self.framed.next().await.ok_or(Error::Closed)??;
            match msg {
                backend::Message::ReadyForQuery(_) => break,
                backend::Message::ErrorResponse(body) => {
                    let mut fields = body.fields();
                    let first = FallibleIterator::next(&mut fields)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?;
                    let msg = first
                        .map(|f: postgres_protocol::message::backend::ErrorField| {
                            String::from_utf8_lossy(f.value_bytes()).into_owned()
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    error = Some(Error::Protocol(msg));
                }
                _ => {}
            }
        }

        if let Some(e) = error {
            return Err(e);
        }
        Ok(())
    }

    pub async fn prepare(&mut self, name: &str, query: &str) -> Result<()> {
        let mut buf = BytesMut::new();
        frontend::parse(name, query, std::iter::empty(), &mut buf)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        frontend::sync(&mut buf);
        self.framed
            .send(buf)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        self.framed
            .flush()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let mut error = None;
        loop {
            let msg = self.framed.next().await.ok_or(Error::Closed)??;
            match msg {
                backend::Message::ParseComplete => {}
                backend::Message::ReadyForQuery(_) => break,
                backend::Message::ErrorResponse(body) => {
                    let mut fields = body.fields();
                    let first = FallibleIterator::next(&mut fields)
                        .map_err(|e: std::io::Error| Error::Protocol(e.to_string()))?;
                    let msg = first
                        .map(|f: postgres_protocol::message::backend::ErrorField| {
                            String::from_utf8_lossy(f.value_bytes()).into_owned()
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    error = Some(Error::Protocol(msg));
                }
                _ => {}
            }
        }

        if let Some(e) = error {
            return Err(e);
        }
        Ok(())
    }
}
