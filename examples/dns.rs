//! a very simplified DNS query to quad9 primary server 9.9.9.9

// NTP protocol implemebtation
use std::net::{Ipv4Addr, UdpSocket};
use std::{error::Error, io::Cursor};

// need this to serialize/deserialize to network
use type2network::{FromNetworkOrder, ToNetworkOrder};
use type2network_derive::{FromNetwork, ToNetwork};

#[derive(Debug, Default, ToNetwork)]
struct Query {
    header: Header,
    question: Question,
}

#[derive(Debug, Default, ToNetwork, FromNetwork)]
struct Header {
    id: u16, // A 16 bit identifier assigned by the program that
    // generates any kind of query.  This identifier is copied
    // the corresponding reply and can be used by the requester
    // to match up replies to outstanding queries.
    flags: u16,
    qd_count: u16, // an unsigned 16 bit integer specifying the number of
    // entries in the question section.
    an_count: u16, // an unsigned 16 bit integer specifying the number of
    // resource records in the answer section.
    ns_count: u16, // an unsigned 16 bit integer specifying the number of name
    // server resource records in the authority records section.
    ar_count: u16, // an unsigned 16 bit integer specifying the number of
                   // resource records in the additional records section.
}

#[derive(Debug, Default, PartialEq, ToNetwork, FromNetwork)]
struct Question {
    qname: [u8; 16],
    qtype: u16,
    qclass: u16,
}

#[derive(Debug, Default, FromNetwork)]
struct Response {
    header: Header,
    question: Question,
    answer: ResourceRecord,
}
#[derive(Debug, Default, FromNetwork)]
struct ResourceRecord {
    name: [u8; 2], // specific due to domain name compression
    r#type: u16,
    class: u16,
    ttl: u32,
    rd_length: u16,
    ipv4: u32, // ip address
}

fn main() -> Result<(), Box<dyn Error>> {
    // prepare query
    let mut q = Query::default();
    q.header.id = 0xfffe;
    q.header.flags = 1;
    q.header.qd_count = 1;

    // RFC1035 representation of www.google.com
    q.question.qname = *b"\x03\x77\x77\x77\x06\x67\x6f\x6f\x67\x6c\x65\x03\x63\x6f\x6d\x00";
    println!("{}", q.question.qname.len());

    // ask for A record
    q.question.qtype = 1;

    // IN class
    q.question.qclass = 1;
    println!("question: {:#?}", q);

    // serialize to send the DNS packet through the wire using UDP
    let mut buffer: Vec<u8> = Vec::new();
    q.serialize_to(&mut buffer)?;

    // bind to an ephemeral local port
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // send packet to destination
    let dest = "9.9.9.9:53";
    socket.send_to(&buffer, dest)?;

    // now receive the request using a sufficient buffer
    let mut recbuf = [0; 512];
    let received = socket.recv(&mut recbuf)?;
    let mut cursor = Cursor::new(&recbuf[..received]);

    // get response
    let mut r = Response::default();
    r.deserialize_from(&mut cursor)?;
    println!("{:#X?}", r);

    // convert u32 ip to ipv4
    let ip = Ipv4Addr::from(r.answer.ipv4);
    println!("ipv4 DNS A record is: {}", ip);

    Ok(())
}
