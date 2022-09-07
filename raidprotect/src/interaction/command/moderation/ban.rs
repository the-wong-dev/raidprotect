//! Ban command.
//!
//! The command allows to ban a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is baned, the action is logged in the database and a message is
//! sent in the guild's logs channel. The baned user receives a pm with the
//! reason of the ban.

use super::util::{
    check_command_permissions, check_user_permissions, get_modal_requirements, get_permissions,
    get_command_data,
};
use raidprotect_model::{cache::model::interaction::PendingSanction, mongodb::modlog::ModlogType};
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    application::{
        component::{text_input::TextInputStyle, ActionRow, Component, TextInput},
        interaction::Interaction,
    },
    guild::Permissions,
    id::{marker::InteractionMarker, Id},
    user::User,
};

use crate::{
    cluster::ClusterState,
    desc_localizations, impl_command_handle,
    interaction::{embed, response::InteractionResponse, util::CustomId},
    translations::Lang,
    util::TextProcessExt,
};

/// Ban command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "ban",
    desc = "Bans a user from the server",
    desc_localizations = "ban_description",
    default_permissions = "BanCommand::default_permissions",
    dm_permission = false
)]
pub struct BanCommand {
    /// Member to ban.
    #[command(rename = "member")]
    pub user: ResolvedUser,
    /// Reason for ban.
    pub reason: Option<String>,
}

impl_command_handle!(BanCommand);
desc_localizations!(ban_description);

impl BanCommand {
    fn default_permissions() -> Permissions {
        Permissions::BAN_MEMBERS
    }

    async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let (guild_id, author_id, lang) = get_command_data(&interaction).await?;
        let user = self.user.resolved;
        let member = match self.user.member {
            Some(member) => member,
            None => return Ok(embed::ban::not_member(user.name, lang)),
        };

        // Set all permissions checks as one function, handle each embed message with a match or something

        // Fetch the author and the bot permissions.
        let (author_permissions, member_permissions, bot_permissions) =
            get_permissions(state, guild_id, author_id, member, &user).await?;

        // Check if the author and the bot have required permissions.
        if let Some(value) = check_user_permissions(
            &member_permissions,
            lang,
            &bot_permissions,
            Permissions::BAN_MEMBERS,
        ) {
            return value;
        }

        // Check if the role hierarchy allow the author and the bot to perform
        // the ban.
        if let Some(value) = check_command_permissions(
            member_permissions,
            author_permissions,
            lang,
            bot_permissions,
        ) {
            return value;
        }

        // Send reason modal.
        let enforce_reason = get_modal_requirements(state, guild_id).await?;

        match self.reason {
            Some(_reason) => Ok(InteractionResponse::EphemeralDeferredMessage),
            None => {
                BanCommand::reason_modal(interaction.id, user, enforce_reason, state, lang).await
            }
        }
    }

    /// Modal that asks the user to enter a reason for the ban.
    ///
    /// This modal is only shown if the user has not specified a reason in the
    /// initial command.
    async fn reason_modal(
        interaction_id: Id<InteractionMarker>,
        user: User,
        enforce_reason: bool,
        state: &ClusterState,
        lang: Lang,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let username = user.name.truncate(15);
        let components = vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "reason".to_string(),
                    label: lang.modal_reason_label().to_string(),
                    max_length: Some(100),
                    min_length: None,
                    placeholder: Some(lang.modal_reason_placeholder().to_string()),
                    required: Some(enforce_reason),
                    style: TextInputStyle::Short,
                    value: None,
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "notes".to_string(),
                    label: lang.modal_notes_label().to_string(),
                    max_length: Some(1000),
                    min_length: None,
                    placeholder: Some(lang.modal_notes_placeholder().to_string()),
                    required: Some(false),
                    style: TextInputStyle::Paragraph,
                    value: None,
                })],
            }),
        ];

        // Add pending component in Redis
        let custom_id = CustomId::new("sanction", interaction_id.to_string());
        let pending = PendingSanction {
            interaction_id,
            kind: ModlogType::Ban,
            user,
        };

        state.redis().set(&pending).await?;

        Ok(InteractionResponse::Modal {
            custom_id: custom_id.to_string(),
            title: lang.modal_ban_title(username),
            components,
        })
    }
}
