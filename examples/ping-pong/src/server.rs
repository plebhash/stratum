use crate::messages::{Ping, Pong, PING_MSG_TYPE, PONG_MSG_TYPE};
use codec_sv2::{Frame, StandardDecoder, StandardSv2Frame};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn start_server(address: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;

    println!("SERVER: Listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    // first, we need to read the ping message

    // initialize decoder
    let mut decoder = StandardDecoder::<Ping>::new();

    // right now, the decoder buffer can only read a frame header
    // because decoder.missing_b is initialized with a header size
    let mut decoder_buf = decoder.writable();

    // read frame header into decoder_buf
    stream.read_exact(&mut decoder_buf)?;

    // this returns an error (MissingBytes), because it only read the header
    // but it also updates decoder.missing_b with the expected frame payload size
    // therefore, we safely ignore the error
    let _ = decoder.next_frame();

    // now, the decoder buffer has the expected size of the frame payload
    let decoder_buf = decoder.writable();

    // read the payload into the decoder_buf
    stream.read_exact(decoder_buf)?;

    // finally read the frame
    let mut frame = decoder.next_frame()?;
    let frame_header = frame
        .get_header()
        .ok_or(anyhow::anyhow!("Failed to read frame header"))?;

    // check message type on header
    if frame_header.msg_type() != PING_MSG_TYPE {
        println!("Received frame was not a Ping. Halting.");
        std::process::exit(1);
    }

    // decode frame payload
    let decoded_payload: Ping = match binary_sv2::from_bytes(frame.payload()) {
        Ok(ping) => ping,
        Err(e) => {
            return Err(anyhow::anyhow!("Could not decode message as a Ping: {}", e));
        }
    };

    // ok, we have successfully received the ping message
    // now it's time to send the pong response

    // we need the ping nonce to create our pong response
    let ping_nonce = decoded_payload.get_nonce();

    println!("SERVER: Received Ping message with nonce: {}", ping_nonce);

    // create Pong message
    let pong_message = Pong::new(ping_nonce)?;

    // create Pong frame
    let pong_frame =
        StandardSv2Frame::<Pong>::from_message(pong_message.clone(), PONG_MSG_TYPE, 0, false)
            .ok_or(anyhow::anyhow!("Failed to create Pong frame"))?;

    // encode Pong frame
    let mut encoder = codec_sv2::Encoder::<Pong>::new();
    let pong_encoded = encoder.encode(pong_frame)?;

    println!(
        "SERVER: Sending Pong to client with nonce: {}",
        pong_message.get_nonce()
    );
    stream.write_all(pong_encoded)?;

    Ok(())
}
