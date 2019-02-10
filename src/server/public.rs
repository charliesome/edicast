use std::io;

use tiny_http::Request;

use super::common::not_found;
use super::Edicast;
use crate::audio::encode;
use crate::fanout::SubscribeError;

pub fn dispatch(req: Request, edicast: &Edicast) -> Result<(), io::Error> {
    let stream_id = match edicast.public_routes.get(req.url()) {
        Some(stream_id) => stream_id,
        None => return not_found(req),
    };

    let content_type = encode::mime_type_from_config(
        &edicast.config.stream[stream_id].codec);

    let stream = match edicast.streams.subscribe_stream(stream_id) {
        Some(stream) => stream,
        None => return not_found(req),
    };

    let mut response = req.into_writer();
    write!(response, "HTTP/1.1 200 OK\r\nContent-Type: {}\r\n\r\n", content_type)?;

    loop {
        match stream.recv() {
            Ok(data) => response.write_all(&data)?,
            Err(SubscribeError::NoPublisher) => {
                // publisher went away, terminate stream
                return Ok(());
            }
        }
    }
}
