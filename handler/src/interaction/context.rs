//! Interaction context.
//!
//! This module contains types used to parse context from received interaction.

use raidprotect_model::ClusterState;
use thiserror::Error;
use twilight_http::client::InteractionClient;
use twilight_model::{
    application::interaction::{application_command::CommandData, ApplicationCommand},
    guild::PartialMember,
    id::{
        marker::{ApplicationMarker, ChannelMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use super::response::{InteractionError, InteractionErrorKind};

/// Context of an [`ApplicationCommand`].
#[derive(Debug)]

pub struct CommandContext {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// ID of the associated application.
    pub application_id: Id<ApplicationMarker>,
    /// Token of the command.
    pub token: String,
    /// Data from the invoked command.
    pub data: CommandData,
    /// The channel the command was triggered from.
    pub channel: Id<ChannelMarker>,
    /// User that triggered the command.
    pub user: User,
    /// If command occurred in a guild, the member that triggered the command.
    pub member: Option<PartialMember>,
    /// The user locale.
    pub locale: String,
}

impl CommandContext {
    /// Initialize a new [`CommandContext`] from an incoming command data
    pub fn from_command(command: ApplicationCommand) -> Result<Self, CommandContextError> {
        // Get user and member data from command context
        let (user, member) = if command.guild_id.is_some() {
            let member = command.member.ok_or(CommandContextError::MissingMember)?;
            let user = member
                .user
                .clone()
                .ok_or(CommandContextError::MissingUser)?;

            (user, Some(member))
        } else {
            let user = command.user.ok_or(CommandContextError::MissingUser)?;

            (user, None)
        };

        Ok(Self {
            id: command.id,
            application_id: command.application_id,
            token: command.token,
            data: command.data,
            channel: command.channel_id,
            user,
            member,
            locale: command.locale,
        })
    }

    /// Get the [`InteractionClient`] associated with the current context.
    pub fn interaction<'state>(&self, state: &'state ClusterState) -> InteractionClient<'state> {
        state.http().interaction(self.application_id)
    }
}

/// Error occurred when initializing a [`CommandContext`].
#[derive(Debug, Error)]
pub enum CommandContextError {
    #[error("missing user data")]
    MissingUser,
    #[error("missing member data")]
    MissingMember,
}

impl InteractionError for CommandContextError {
    const INTERACTION_NAME: &'static str = "context";

    fn into_error(self) -> InteractionErrorKind {
        InteractionErrorKind::internal(self)
    }
}