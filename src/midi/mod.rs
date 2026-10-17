use std::{ffi::CString, str::FromStr};

use alsa::seq::{Addr, PortCap, PortSubscribe, PortType};

pub struct MidiClient {
    seq: alsa::Seq,
    port_id: i32,
}

impl MidiClient {
    pub fn new(name: &str) -> Self {
        let seq = alsa::Seq::open(None, None, false).unwrap();
        seq.set_client_name(&CString::from_str(name).unwrap())
            .unwrap();
        let port_id = seq
            .create_simple_port(
                &CString::from_str(name).unwrap(),
                PortCap::WRITE | PortCap::SUBS_WRITE | PortCap::READ | PortCap::SUBS_READ,
                PortType::APPLICATION,
            )
            .unwrap();

        Self { seq, port_id }
    }

    pub fn connect(&mut self, port_spec: &str) {
        let (src_id, src_port_id) = port_spec
            .split_once(":")
            .expect("failed to parse MIDI port");
        let src_id: i32 = src_id.parse().expect("failed to parse MIDI source ID");
        let src_port_id: i32 = src_port_id
            .parse()
            .expect("failed to parse MIDI source port ID");

        let subscribe = PortSubscribe::empty().unwrap();
        subscribe.set_sender(Addr {
            client: src_id,
            port: src_port_id,
        });
        subscribe.set_dest(Addr {
            client: self.seq.client_id().unwrap(),
            port: self.port_id,
        });

        self.seq.subscribe_port(&subscribe).unwrap();
    }

    pub fn input(&self) -> alsa::seq::Input<'_> {
        self.seq.input()
    }
}
