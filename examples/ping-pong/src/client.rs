use crate::messages::{Ping, Pong, PING_MSG_TYPE, PONG_MSG_TYPE};
use codec_sv2::{Frame, StandardDecoder, StandardSv2Frame};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn start_client(address: &str) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(address)?;

    println!("CLIENT: Connected to server on {}", address);

    // create Ping message
    let ping_message = Ping::new()?;
    let ping_nonce = ping_message.get_nonce();

    // create Ping frame
    let ping_frame =
        StandardSv2Frame::<Ping>::from_message(ping_message.clone(), PING_MSG_TYPE, 0, false)
            .ok_or(anyhow::anyhow!("Failed to create ping frame"))?;

    // encode Ping frame
    let mut encoder = codec_sv2::Encoder::<Ping>::new();
    let ping_encoded = encoder.encode(ping_frame)?;

    println!("CLIENT: Sending Ping to server with nonce: {}", ping_nonce);
    stream.write_all(ping_encoded)?;

    // ok, we have successfully sent the ping message
    // now it's time to receive and verify the pong response

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
    if frame_header.msg_type() != PONG_MSG_TYPE {
        return Err(anyhow::anyhow!("CLIENT: Received frame was not a Pong."));
    }

    // decode frame payload
    let decoded_payload: Pong = match binary_sv2::from_bytes(frame.payload()) {
        Ok(pong) => pong,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "CLIENT: Could not decode message as a Ping: {}",
                e
            ));
        }
    };

    // check if nonce is the same as ping
    let pong_nonce = decoded_payload.get_nonce();
    if ping_nonce == pong_nonce {
        println!(
            "CLIENT: Received Pong with identical nonce as Ping: {}",
            pong_nonce
        );
    } else {
        return Err(anyhow::anyhow!(
            "CLIENT: nonce on received pong was not the same as ping"
        ));
    }

    Ok(())
}
