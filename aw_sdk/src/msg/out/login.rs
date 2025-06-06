use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{AwInstance, SdkError, SdkResult, instance::Session};

pub fn login(instance: &mut AwInstance, params: LoginParams) -> SdkResult<LoginResponse> {
    let packet = {
        let mut p = AWPacket::new(PacketType::Login);
        match params {
            LoginParams::Bot {
                name,
                owner_id,
                privilege_password,
                application,
            } => {
                p.add_uint(VarID::LoginID, owner_id);
                p.add_int(VarID::BrowserBuild, 93);
                p.add_int(VarID::UserType, 3); // 3 = Bot
                p.add_int(VarID::BrowserVersion, 0x50001);
                p.add_string(VarID::LoginUsername, name);
                p.add_string(VarID::PrivilegePassword, privilege_password);
                p.add_string(VarID::Application, application);
                p.add_int(VarID::VolumeSerial, 255);
            }
            LoginParams::Player { name, password } => {
                p.add_int(VarID::BrowserBuild, 2007);
                p.add_int(VarID::UserType, 2); // 2 = Player
                p.add_int(VarID::BrowserVersion, 0x50001);
                p.add_string(VarID::LoginUsername, name);
                p.add_string(VarID::Password, password);
                p.add_int(VarID::VolumeSerial, 255);
            }
        }
        p
    };

    instance.uni.send(packet);

    let response = instance
        .uni
        .wait_for_packet(PacketType::Login, Some(instance.timeout))
        .ok_or(SdkError::Timeout)?;

    let reason_code = response
        .get_int(VarID::ReasonCode)
        .ok_or_else(|| SdkError::missing_field("ReasonCode"))?;

    let reason_code =
        ReasonCode::try_from(reason_code).map_err(|_| SdkError::protocol("Invalid reason code"))?;

    if reason_code == ReasonCode::Success {
        eprintln!("Login response: {response:?}");

        let session_id = response
            .get_uint(VarID::SessionID)
            .ok_or_else(|| SdkError::missing_field("SessionID"))?;
        let login_id = response.get_uint(VarID::LoginID);

        let session = Session {
            session_id,
            login_id,
        };

        instance.session = Some(session);

        let name = response
            .get_string(VarID::CitizenName)
            .ok_or_else(|| SdkError::missing_field("CitizenName"))?;

        let login_response = LoginResponse { name };

        Ok(login_response)
    } else {
        Err(SdkError::ActiveWorldsError(reason_code))
    }
}

pub enum LoginParams {
    Bot {
        name: String,
        owner_id: u32,
        privilege_password: String,
        application: String,
    },
    Player {
        name: String,
        password: String,
    },
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub name: String,
}
