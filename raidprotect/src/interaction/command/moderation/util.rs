/*
    Requirements:
    1. User is Moderator
    2. Bot Checks permissions for User and Target
    3. If no reason given, show modal. If enforce_reason is True, reason is required.
    4. Target gets message about action being taken
    5. Bot performs command
    6. Message sent to Guild Logs and stored in DB
    7. User gets message about action being taken.
*/

use crate::{
    cluster::ClusterState,
    interaction::{embed, response::InteractionResponse, util::InteractionExt},
    translations::Lang,
};
use anyhow::{Error, Context};
use twilight_model::{
    application::interaction::Interaction, guild::Permissions, id::Id, user::User,
};

pub async fn init_command(
    interaction: &Interaction,
) -> Result<
    (
        Id<twilight_model::id::marker::GuildMarker>,
        Id<twilight_model::id::marker::UserMarker>,
        Lang,
    ),
    anyhow::Error,
> {
    let guild_id = interaction.guild()?.id;
    let author_id = interaction.author_id().context("missing author_id")?;
    let lang = interaction.locale()?;
    Ok((guild_id, author_id, lang))
}

/* Handle Permissions */
pub async fn get_permissions<'a>(
    state: &'a ClusterState,
    guild_id: Id<twilight_model::id::marker::GuildMarker>,
    author_id: Id<twilight_model::id::marker::UserMarker>,
    member: twilight_model::application::interaction::application_command::InteractionMember,
    user: &User,
) -> Result<
    (
        raidprotect_model::cache::permission::CachePermissions<'a>,
        raidprotect_model::cache::permission::CachePermissions<'a>,
        raidprotect_model::cache::permission::CachePermissions<'a>,
    ),
    anyhow::Error,
> {
    let permissions = state.redis().permissions(guild_id).await?;
    let author_permissions = permissions.member(author_id, &member.roles).await?;
    let member_permissions = permissions.member(user.id, &member.roles).await?;
    let bot_permissions = permissions.current_member().await?;
    Ok((author_permissions, member_permissions, bot_permissions))
}

pub fn check_user_permissions(
    member_permissions: &raidprotect_model::cache::permission::CachePermissions,
    lang: Lang,
    bot_permissions: &raidprotect_model::cache::permission::CachePermissions,
    command_permissions: Permissions,
) -> Option<Result<InteractionResponse, anyhow::Error>> {
    if member_permissions.is_owner() {
        return Some(Ok(embed::kick::member_owner(lang)));
    }
    if !bot_permissions.guild().contains(command_permissions) {
        return Some(Ok(embed::kick::bot_missing_permission(lang)));
    }
    None
}

pub fn check_command_permissions(
    member_permissions: raidprotect_model::cache::permission::CachePermissions,
    author_permissions: raidprotect_model::cache::permission::CachePermissions,
    lang: Lang,
    bot_permissions: raidprotect_model::cache::permission::CachePermissions,
) -> Option<Result<InteractionResponse, anyhow::Error>> {
    let member_highest_role = member_permissions.highest_role();
    if member_highest_role >= author_permissions.highest_role() {
        return Some(Ok(embed::kick::user_hierarchy(lang)));
    }
    if member_highest_role >= bot_permissions.highest_role() {
        return Some(Ok(embed::kick::bot_hierarchy(lang)));
    }
    None
}

/* Send Messages */
pub fn _message_log() {}

pub fn _message_target() {}

pub fn _message_user() {}

/**
Get enforce_reason value from MongoDB for this Guild.
*/
pub async fn reason_enforced(
    state: &ClusterState,
    guild_id: Id<twilight_model::id::marker::GuildMarker>,
) -> Result<bool, Error> {
    let required = state
        .mongodb()
        .get_guild_or_create(guild_id)
        .await?
        .moderation
        .enforce_reason;
    Ok(required)
}
pub fn _perform_command() {}
