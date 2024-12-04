use clap::error::{ContextKind, ContextValue, ErrorKind};
use serde_json::{Map, Value};
use std::io::Write;
use termcolor::WriteColor;
use uuid::Uuid;

pub struct StyledStr {
    messages: Vec<(Option<Style>, String)>,
}

impl StyledStr {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push_str(&mut self, style: Option<Style>, msg: String) {
        if !msg.is_empty() {
            self.messages.push((style, msg));
        }
    }

    pub fn print_err(&self) -> std::io::Result<()> {
        let bufwtr = termcolor::BufferWriter::stderr(termcolor::ColorChoice::Always);
        let mut buffer = bufwtr.buffer();

        for (style, message) in &self.messages {
            let mut color = termcolor::ColorSpec::new();
            match style {
                Some(Style::Success) => {
                    color.set_fg(Some(termcolor::Color::Green));
                }
                Some(Style::Warning) => {
                    color.set_fg(Some(termcolor::Color::Yellow));
                }
                Some(Style::Error) => {
                    color.set_fg(Some(termcolor::Color::Red));
                    color.set_bold(true);
                }
                None => {}
            }

            buffer.set_color(&color)?;
            write!(buffer, "{message}")?;
        }

        write!(buffer, "\r\n")?;
        bufwtr.print(&buffer)?;

        Ok(())
    }

    pub fn print_data_err(&self) -> ! {
        self.print_err().unwrap();

        // DATAERR
        std::process::exit(65)
    }

    pub fn print_success(&self) -> ! {
        self.print_err().unwrap();

        // SUCCESS
        std::process::exit(0)
    }
}

pub enum Style {
    Success,
    Warning,
    Error,
}

pub fn maybe_json(data: Option<String>) -> Option<Map<String, Value>> {
    if let Some(json) = data {
        match serde_json::from_str(json.as_str()) {
            Ok(json_data) => Some(json_data),
            Err(_e) => None,
        }
    } else {
        None
    }
}

fn prn_error(cmd: &clap::Command, arg: Option<&clap::Arg>, error: &str) -> clap::Error {
    let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
    if let Some(arg) = arg {
        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.to_string()),
        );
    }
    err.insert(
        ContextKind::InvalidValue,
        ContextValue::String(error.to_string()),
    );
    err
}

#[derive(Clone, PartialEq, Debug)]
pub enum PRNType {
    APIKey,
    Artifact,
    ArtifactVersion,
    AuditLog,
    Binary,
    BinaryPart,
    BinarySignature,
    Bundle,
    BundleOverride,
    CACertificate,
    Cohort,
    Deployment,
    Device,
    DeviceCertificate,
    Event,
    Firmware,
    OrgUser,
    Organization,
    Product,
    Release,
    ReleaseClaim,
    SigningKey,
    Tunnel,
    User,
    WebConsoleShell,
    Webhook,
    UserToken,
}

impl TryFrom<String> for PRNType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<PRNType, Self::Error> {
        match value.as_str() {
            "api_key" => Ok(Self::APIKey),
            "artifact" => Ok(Self::Artifact),
            "artifact_version" => Ok(Self::ArtifactVersion),
            "audit_log" => Ok(Self::AuditLog),
            "binary" => Ok(Self::Binary),
            "binary_part" => Ok(Self::BinaryPart),
            "binary_signature" => Ok(Self::BinarySignature),
            "bundle" => Ok(Self::Bundle),
            "bundle_override" => Ok(Self::BundleOverride),
            "ca_certificate" => Ok(Self::CACertificate),
            "cohort" => Ok(Self::Cohort),
            "deployment" => Ok(Self::Deployment),
            "device" => Ok(Self::Device),
            "device_certificate" => Ok(Self::DeviceCertificate),
            "event" => Ok(Self::Event),
            "firmware" => Ok(Self::Firmware),
            "org_user" => Ok(Self::OrgUser),
            "organization" => Ok(Self::Organization),
            "product" => Ok(Self::Product),
            "release" => Ok(Self::Release),
            "release_claim" => Ok(Self::ReleaseClaim),
            "signing_key" => Ok(Self::SigningKey),
            "tunnel" => Ok(Self::Tunnel),
            "user" => Ok(Self::User),
            "web_console_shell" => Ok(Self::WebConsoleShell),
            "webhook" => Ok(Self::Webhook),
            "user_token" => Ok(Self::UserToken),
            _ => Err("Invalid PRN type"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct PRNValueParser(PRNType);

impl PRNValueParser {
    pub fn new(prn_type: PRNType) -> Self {
        Self(prn_type)
    }
}

impl clap::builder::TypedValueParser for PRNValueParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value: String = value.to_str().unwrap().to_owned();

        let mut split = value.split(':').fuse();

        let prn_length = split.clone().count();

        if !(3..=5).contains(&prn_length) {
            return Err(prn_error(cmd, arg, "Invalid PRN"));
        }

        if split.next().is_some_and(|x| x != "prn") {
            return Err(prn_error(cmd, arg, "Invalid PRN"));
        }

        if split.next().is_some_and(|x| x != "1") {
            return Err(prn_error(cmd, arg, "Invalid PRN"));
        }

        match prn_length {
            3 => {
                // organization prn only
                if self.0 != PRNType::Organization {
                    return Err(prn_error(cmd, arg, "Invalid PRN type"));
                }
                // the uuid has to be valid
                if Uuid::try_parse(split.next().unwrap()).is_err() {
                    return Err(prn_error(
                        cmd,
                        arg,
                        "Invalid PRN UUID, expected 'organization' UUID in PRN",
                    ));
                }

                0
            }
            4 => {
                // user or user token
                if self.0 != PRNType::User || self.0 != PRNType::UserToken {
                    return Err(prn_error(cmd, arg, "Invalid PRN type"));
                }

                let prn_type = PRNType::try_from(split.next().unwrap().to_string());

                if prn_type.is_err() {
                    return Err(prn_error(cmd, arg, "Invalid PRN type"));
                }

                let prn_type = prn_type.unwrap();

                if prn_type != PRNType::User || prn_type != PRNType::UserToken {
                    return Err(prn_error(
                        cmd,
                        arg,
                        "Invalid PRN type, expected 'user' or 'user_token' PRN",
                    ));
                }

                0
            }
            5 => {
                // the org uuid has to be valid
                if Uuid::try_parse(split.next().unwrap()).is_err() {
                    return Err(prn_error(
                        cmd,
                        arg,
                        "Invalid PRN UUID, expected valid UUID in PRN",
                    ));
                }

                let prn_type = PRNType::try_from(split.next().unwrap().to_string());

                if prn_type.is_err() {
                    return Err(prn_error(cmd, arg, "Invalid PRN type"));
                }

                let prn_type = prn_type.unwrap();

                if self.0 != prn_type {
                    return Err(prn_error(
                        cmd,
                        arg,
                        format!("Invalid PRN type, expected '{:#?}' PRN", self.0).as_str(),
                    ));
                }

                // the uuid has to be valid
                if Uuid::try_parse(split.next().unwrap()).is_err() {
                    return Err(prn_error(
                        cmd,
                        arg,
                        "Invalid PRN UUID, expected valid UUID in PRN",
                    ));
                }

                0
            }
            _ => return Err(prn_error(cmd, arg, "Invalid PRN")),
        };

        Ok(value)
    }
}
