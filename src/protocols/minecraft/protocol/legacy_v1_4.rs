
use crate::GDResult;
use crate::bufferer::{Bufferer, Endianess};
use crate::GDError::{PacketBad, ProtocolFormat};
use crate::protocols::minecraft::{LegacyGroup, JavaResponse, Server};
use crate::protocols::minecraft::protocol::legacy_v1_6::LegacyV1_6;
use crate::protocols::types::TimeoutSettings;
use crate::socket::{Socket, TcpSocket};
use crate::utils::error_by_expected_size;

pub struct LegacyV1_4 {
    socket: TcpSocket
}

impl LegacyV1_4 {
    fn new(address: &str, port: u16, timeout_settings: Option<TimeoutSettings>) -> GDResult<Self> {
        let socket = TcpSocket::new(address, port)?;
        socket.apply_timeout(timeout_settings)?;

        Ok(Self {
            socket
        })
    }

    fn send_initial_request(&mut self) -> GDResult<()> {
        self.socket.send(&[0xFE, 0x01])
    }

    fn get_info(&mut self) -> GDResult<JavaResponse> {
        self.send_initial_request()?;

        let mut buffer = Bufferer::new_with_data(Endianess::Big, &self.socket.receive(None)?);

        if buffer.get_u8()? != 0xFF {
            return Err(ProtocolFormat);
        }

        let length = buffer.get_u16()? * 2;
        error_by_expected_size((length + 3) as usize, buffer.data_length())?;

        if LegacyV1_6::is_protocol(&mut buffer)? {
            return LegacyV1_6::get_response(&mut buffer);
        }

        let packet_string = buffer.get_string_utf16()?;

        let split: Vec<&str> = packet_string.split("§").collect();
        error_by_expected_size(3, split.len())?;

        let description = split[0].to_string();
        let online_players = split[1].parse()
            .map_err(|_| PacketBad)?;
        let max_players = split[2].parse()
            .map_err(|_| PacketBad)?;

        Ok(JavaResponse {
            version_name: "1.4+".to_string(),
            version_protocol: -1,
            players_maximum: max_players,
            players_online: online_players,
            players_sample: None,
            description,
            favicon: None,
            previews_chat: None,
            enforces_secure_chat: None,
            server_type: Server::Legacy(LegacyGroup::V1_4)
        })
    }

    pub fn query(address: &str, port: u16, timeout_settings: Option<TimeoutSettings>) -> GDResult<JavaResponse> {
        LegacyV1_4::new(address, port, timeout_settings)?.get_info()
    }
}
