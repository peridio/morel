mod artifacts;
mod binaries;
mod binary_parts;
mod binary_signatures;
mod ca_certificates;
mod cohorts;
mod deployments;
mod device_certificates;
mod devices;
mod firmwares;
mod organization;
mod products;
mod signing_keys;
mod upgrade;
mod users;
use clap::Parser;
use crate:: utils::Style;
use crate:: utils::StyledStr;
use crate::GlobalOptions;

#[derive(Parser, Debug)]
pub struct Command<T>
where
    T: Parser + clap::Args,
{
    #[command(flatten)]
    inner: T,
}

#[derive(clap::Subcommand, Debug)]
pub enum CliCommands {
    #[command(flatten)]
    ApiCommand(ApiCommand),
    #[command()]
    Upgrade(upgrade::UpgradeCommand),
}

#[derive(clap::Subcommand, Debug)]
pub enum ApiCommand {
    #[command(subcommand)]
    Artifacts(artifacts::ArtifactsCommand),
    #[command(subcommand)]
    Binaries(binaries::BinariesCommand),
    #[command(subcommand)]
    BinaryParts(binary_parts::BinaryPartsCommand),
    #[command(subcommand)]
    BinarySignatures(binary_signatures::BinarySignaturesCommand),
    #[command(subcommand)]
    Cohorts(cohorts::CohortsCommand),
    #[command(subcommand)]
    CaCertificates(ca_certificates::CaCertificatesCommand),
    #[command(subcommand)]
    Deployments(deployments::DeploymentsCommand),
    #[command(subcommand)]
    Devices(devices::DevicesCommand),
    #[command(subcommand)]
    DeviceCertificates(device_certificates::DeviceCertificatesCommand),
    #[command(subcommand)]
    Firmwares(firmwares::FirmwaresCommand),
    #[command(subcommand)]
    Organizations(organization::OrganizationCommand),
    #[command(subcommand)]
    Products(products::ProductsCommand),
    #[command(subcommand)]
    SigningKeys(signing_keys::SigningKeysCommand),
    #[command(subcommand)]
    Users(users::UsersCommand),
}

impl CliCommands {
    pub(crate) async fn run(self, global_options: GlobalOptions) -> Result<(), crate::Error> {
        match self {
            CliCommands::ApiCommand(api) => {
                // require api key
                let mut error_vec = Vec::new();

                if global_options.api_key.is_none() {
                    error_vec.push("--api-key".to_owned());
                }

                // require organization name
                if global_options.organization_name.is_none() {
                    error_vec.push("--organization-name".to_owned());
                }

                if !error_vec.is_empty() {
                    let mut error = StyledStr::new();

                    error.push_str(Some(Style::Error), "error: ".to_string());
                    error.push_str(
                        None,
                        "The following arguments are required at the global level:\r\n".to_string(),
                    );
                    for error_msg in error_vec.iter() {
                        error.push_str(Some(Style::Success), format!("\t{error_msg}\r\n"));
                    }
                    error.print_data_err();
                }

                match api {
                    ApiCommand::Artifacts(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Binaries(cmd) => cmd.run(global_options).await?,
                    ApiCommand::BinaryParts(cmd) => cmd.run(global_options).await?,
                    ApiCommand::BinarySignatures(cmd) => cmd.run(global_options).await?,
                    ApiCommand::CaCertificates(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Cohorts(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Deployments(cmd) => cmd.run(global_options).await?,
                    ApiCommand::DeviceCertificates(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Devices(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Firmwares(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Organizations(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Products(cmd) => cmd.run(global_options).await?,
                    ApiCommand::SigningKeys(cmd) => cmd.run(global_options).await?,
                    ApiCommand::Users(cmd) => cmd.run(global_options).await?,
                }
            }
            CliCommands::Upgrade(cmd) => cmd.run().await?,
        };

        Ok(())
    }
}
