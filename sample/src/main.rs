// NTP protocol implemebtation
use std::{error::Error, io::Cursor};
use std::net::UdpSocket;

// need this to serialize/deserialize to network
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};

// 1                   2                   3
// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9  0  1
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |LI | VN  |Mode |    Stratum    |     Poll      |   Precision    |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                          Root  Delay                           |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                       Root  Dispersion                         |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                     Reference Identifier                       |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                                |
// |                    Reference Timestamp (64)                    |
// |                                                                |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                                |
// |                    Originate Timestamp (64)                    |
// |                                                                |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                                |
// |                     Receive Timestamp (64)                     |
// |                                                                |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                                |
// |                     Transmit Timestamp (64)                    |
// |                                                                |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                 Key Identifier (optional) (32)                 |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                                |
// |                                                                |
// |                 Message Digest (optional) (128)                |
// |                                                                |
// |                                                                |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct Header {
    values: u8,  // includes Leap Indicator, Version Number & Mode
    stratum: u8,
    poll: u8,
    precision: u8,
}

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct Timestamp {
    seconds: u32,
    fraction: u32,
}

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct SNTPPacket {
    header: Header,
    root_delay: u32,
    root_dispersion: u32,
    ref_id: u32,
    ref_tms: Timestamp,
    orig_tms: Timestamp,
    recv_tms: Timestamp,
    xmit_tms: Timestamp,
    id: Option<u32>,
    digest: Option<u128>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // fill client packet
    let mut ntp = SNTPPacket::default();

    // set version number to 4 and mode to 3 (client)
    ntp.header.values = 4 << 3 | 3;

    // serialize to send the SNTP packet through the wire using UDP
    let mut buffer: Vec<u8> = Vec::new();
    ntp.serialize_to(&mut buffer)?;    
    
    // bind to an ephemeral local port
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // send packet to destination
    //let dest = format!("{}:123", "fr.pool.ntp.org");
    let dest = "fr.pool.ntp.org:123";
    socket.send_to(&buffer, dest)?;

    // now receive the request using a sufficient buffer
    let mut recbuf = [0; 512];
    let received = socket.recv(&mut recbuf)?;
    let mut cursor = Cursor::new(&recbuf[..received]);

    // get response
    ntp.deserialize_from(&mut cursor)?;
    println!("{:?}", ntp);

    Ok(()) 

}
