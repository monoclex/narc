//! Copy & pasted code from `serenity_utils` because it doesn't support the
//! git version of serenity currently.

// ISC License
// Copyright (c) 2020, AriusX7
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! A library to provide additional utilies for Discord bots created with [`serenity`].
//!
//! This library provides implementations to easily:
//! - Convert a string to [`serenity`]'s guild-specific models.
//! - Get user response using message or reaction prompts.
//! - Display paginated reaction-based messages/menus.
//! - Format text in different ways before sending.
//! - Create embeds and messages with field access.
//!
//! See module level documentation for in-depth info about the utilities
//! provided by this crate.
//!
//! ## Installation and Usage
//!
//! To use this crate, add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! serenity_utils = "0.6.1"
//! ```
//!
//! **Note:** This crate only supports [`serenity`]'s async versions.
//!
//! ## Examples
//!
//! Examples detailing and explaining usage of the basic functionality of the
//! library can be found in the [`examples`] directory.
//!
//! [`serenity`]: https://github.com/serenity-rs/serenity
//! [`examples`]: https://github.com/AriusX7/serenity-utils/tree/current/examples

#[doc(inline)]
pub use error::Error;

pub mod prompt {
    //! Prompts to get user's response interactively.
    //!
    //! ## Examples
    //!
    //! This library provides two types of prompts: message-based and reaction-based.
    //! An example for both is given below.
    //!
    //! ### Message Prompt
    //!
    //! ```
    //! # use serenity::{
    //! #    model::prelude::{ChannelId, Message},
    //! #    prelude::Context,
    //! # };
    //! # use serenity_utils::{prompt::message_prompt_content, Error};
    //! #
    //! async fn mprompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
    //!     let prompt_msg = ChannelId(7).say(&ctx.http, "What is your favourite colour?").await?;
    //!
    //!     // User's optional response to the message.
    //!     let optional_content = message_prompt_content(ctx, &prompt_msg, &msg.author, 30.0).await;
    //!
    //!     Ok(())
    //! }
    //! ```
    //!
    //! ### Reaction Prompt
    //!
    //! ```
    //! # use serenity::{
    //! #    model::prelude::{ChannelId, Message},
    //! #    prelude::Context,
    //! # };
    //! # use serenity_utils::{prompt::yes_or_no_prompt, Error};
    //! #
    //! async fn rprompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
    //!     let prompt_msg = ChannelId(7).say(&ctx.http, "Is red your favourite colour?").await?;
    //!
    //!     // Result of user's reaction to the prompt.
    //!     let result = yes_or_no_prompt(ctx, &prompt_msg, &msg.author, 30.0).await?;
    //!
    //!     Ok(())
    //! }
    //! ```
    //!
    //! For more in-depth usage and examples, see individual functions.

    #[doc(inline)]
    pub use message::*;
    #[doc(inline)]
    pub use reaction::*;

    pub mod message {
        //! Prompts to get a user's response via a message.
        //!
        //! ## Example
        //!
        //! ```
        //! # use serenity::{
        //! #    model::prelude::{ChannelId, Message},
        //! #    prelude::Context,
        //! # };
        //! # use serenity_utils::{prompt::message_prompt_content, Error};
        //! #
        //! async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        //!     // Assuming `channel_id` is bound.
        //!     let prompt_msg = ChannelId(7).say(&ctx.http, "What is your favourite colour?").await?;
        //!
        //!     // User's optional response to the message.
        //!     let optional_content = message_prompt_content(ctx, &prompt_msg, &msg.author, 30.0).await;
        //!
        //!     Ok(())
        //! }
        //! ```

        use serenity::{
            model::prelude::{Message, User},
            prelude::Context,
        };
        use std::time::Duration;

        /// Creates a message prompt to get the next message a user sends.
        ///
        /// Only messages sent in the channel of the original message are considered.
        /// The bot waits for a message for `timeout` seconds only. `None` is returned
        /// if the user does not send another message.
        ///
        /// ## Example
        ///
        /// ```
        /// # use serenity::{
        /// #    model::prelude::{ChannelId, Message},
        /// #    prelude::Context,
        /// # };
        /// # use serenity_utils::{prompt::message_prompt, Error};
        /// #
        /// async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        ///     // Assuming `channel_id` is bound.
        ///     let prompt_msg = ChannelId(7).say(&ctx.http, "What is your favourite colour?").await?;
        ///
        ///     // Optional `Message` object of user's response to the message.
        ///     let optional_msg = message_prompt(ctx, &prompt_msg, &msg.author, 30.0).await;
        ///
        ///     Ok(())
        /// }
        /// ```
        ///
        /// See [`message_prompt_content`] if you only need the message's content.
        ///
        /// [`message_prompt_content`]: message_prompt_content()
        pub async fn message_prompt(
            ctx: &Context,
            msg: &Message,
            user: &User,
            timeout: f32,
        ) -> Option<Message> {
            user.await_reply(&ctx)
                .channel_id(msg.channel_id)
                .timeout(Duration::from_secs_f32(timeout))
                .await
                .map(|m| m.as_ref().clone())
        }

        /// Creates a message prompt to get the content of the next message a user sends.
        ///
        /// Only messages sent in the channel of the original message are considered.
        /// The bot waits for a message for `timeout` seconds only. `None` is returned
        /// if the user does not send another message.
        ///
        /// ## Example
        ///
        /// ```
        /// # use serenity::{
        /// #    model::prelude::{ChannelId, Message},
        /// #    prelude::Context,
        /// # };
        /// # use serenity_utils::{prompt::message_prompt_content, Error};
        /// #
        /// async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        ///     // Assuming `channel_id` is bound.
        ///     let prompt_msg = ChannelId(7).say(&ctx.http, "What is your favourite colour?").await?;
        ///
        ///     // User's optional response to the message.
        ///     let optional_content = message_prompt_content(ctx, &prompt_msg, &msg.author, 30.0).await;
        ///
        ///     Ok(())
        /// }
        /// ```
        ///
        /// See [`message_prompt`] if you need the whole message object.
        ///
        /// [`message_prompt`]: message_prompt()
        pub async fn message_prompt_content(
            ctx: &Context,
            msg: &Message,
            user: &User,
            timeout: f32,
        ) -> Option<String> {
            user.await_reply(&ctx)
                .channel_id(msg.channel_id)
                .timeout(Duration::from_secs_f32(timeout))
                .await
                .map(|m| m.content.clone())
        }
    }

    pub mod reaction {
        //! Prompts to get a user's response via a reaction.
        //!
        //! ## Example
        //!
        //! ```
        //! # use serenity::{
        //! #    model::prelude::{ChannelId, Message},
        //! #    prelude::Context,
        //! # };
        //! # use serenity_utils::{prompt::yes_or_no_prompt, Error};
        //! #
        //!
        //! async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        //!     let prompt_msg = ChannelId(7).say(&ctx.http, "What is your favourite colour?").await?;
        //!
        //!     // Result of user's reaction to the prompt.
        //!     let result = yes_or_no_prompt(ctx, &prompt_msg, &msg.author, 30.0).await?;
        //!
        //!     Ok(())
        //! }
        //! ```

        use crate::serenity_utils::{error::Error, misc::add_reactions};
        use serenity::{
            collector::ReactionAction,
            futures::StreamExt,
            model::prelude::{Message, ReactionType, User},
            prelude::Context,
        };
        use std::time::Duration;

        /// Creates a reaction prompt to get user's reaction.
        ///
        /// Reactions are collected on the specified message. Only messages sent by `user`
        /// are considered. Reactions are only considered for `timeout` seconds.
        ///
        /// ## Example
        ///
        /// ```
        /// # use serenity::{
        /// #    model::prelude::{ChannelId, Message, ReactionType},
        /// #    prelude::Context,
        /// # };
        /// # use serenity_utils::{prompt::reaction_prompt, Error};
        /// #
        /// async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        ///     // Emojis for the prompt.
        ///     let emojis = [
        ///         ReactionType::from('🐶'),
        ///         ReactionType::from('🐱'),
        ///     ];
        ///
        ///     let prompt_msg = ChannelId(7).say(&ctx.http, "Dogs or cats?").await?;
        ///
        ///     // Creates the prompt and returns the result. Because of `reaction_prompt`'s
        ///     // return type, you can use the `?` operator to get the result.
        ///     // The `Ok()` value is the selected emoji's index (wrt the `emojis` slice)
        ///     // and the emoji itself. We don't require the emoji here, so we ignore it.
        ///     let (idx, _) = reaction_prompt(
        ///         ctx,
        ///         &prompt_msg,
        ///         &msg.author,
        ///         &emojis,
        ///         30.0
        ///     )
        ///     .await?;
        ///
        ///     if idx == 0 {
        ///         // Dogs!
        ///     } else {
        ///         // Cats!
        ///     }
        ///
        ///     Ok(())
        /// }
        /// ```
        ///
        /// ## Errors
        ///
        /// Returns [`Error::SerenityError`] if cache is enabled and the current
        /// user does not have the required permissions to add reactions.
        ///
        /// Returns [`Error::TimeoutError`] if user does not react at all.
        ///
        /// [`Error::SerenityError`]: crate::error::Error::SerenityError
        /// [`Error::TimeoutError`]: crate::error::Error::TimeoutError
        pub async fn reaction_prompt(
            ctx: &Context,
            msg: &Message,
            user: &User,
            emojis: &[ReactionType],
            timeout: f32,
        ) -> Result<(usize, ReactionType), Error> {
            add_reactions(ctx, msg, emojis.to_vec()).await?;

            let mut collector = user
                .await_reactions(&ctx)
                .message_id(msg.id)
                .timeout(Duration::from_secs_f32(timeout))
                .await;

            while let Some(action) = collector.next().await {
                if let ReactionAction::Added(reaction) = action.as_ref() {
                    if emojis.contains(&reaction.emoji) {
                        return Ok((
                            emojis.iter().position(|p| p == &reaction.emoji).unwrap(),
                            reaction.emoji.clone(),
                        ));
                    }
                }
            }

            Err(Error::TimeoutError)
        }

        /// A special reaction prompt to check if user reacts with yes or no.
        ///
        /// ✅ is used for yes and ❌ is used for no.
        ///
        /// This function behaves in same way as [`reaction_prompt`] except for the
        /// return type. If the user reacts with the yes emoji, the Ok value is `true`.
        /// It user reacts with no emoji, the value is `false`.
        ///
        /// ## Example
        ///
        /// ```
        /// # use serenity::{
        /// #    model::prelude::{ChannelId, Message, ReactionType},
        /// #    prelude::Context,
        /// # };
        /// # use serenity_utils::{prompt::yes_or_no_prompt, Error};
        /// #
        /// async fn prompt(ctx: &Context, msg: &Message) -> Result<(), Error> {
        ///     let prompt_msg = ChannelId(7).say(&ctx.http, "Are you a bot?").await?;
        ///
        ///     // Creates a yes/no prompt and returns the result.
        ///     let result = yes_or_no_prompt(
        ///         ctx,
        ///         &prompt_msg,
        ///         &msg.author,
        ///         30.0
        ///     )
        ///     .await?;
        ///
        ///     if result {
        ///         // Is a bot!
        ///     } else {
        ///         // Not a bot!
        ///     }
        ///
        ///     Ok(())
        /// }
        /// ```
        ///
        /// ## Errors
        ///
        /// It can return the same errors as [`reaction_prompt`].
        ///
        /// [`reaction_prompt`]: reaction_prompt()
        pub async fn yes_or_no_prompt(
            ctx: &Context,
            msg: &Message,
            user: &User,
            timeout: f32,
        ) -> Result<bool, Error> {
            let emojis = [ReactionType::from('✅'), ReactionType::from('❌')];

            reaction_prompt(ctx, msg, user, &emojis, timeout)
                .await
                .map(|(i, _)| i == 0)
        }
    }
}

pub mod error {
    use serenity::Error as SerenityError;
    use std::{
        borrow::Cow,
        error::Error as StdError,
        fmt::{self, Display, Formatter},
    };

    /// A common error type for all functions and methods of the library.
    ///
    /// It can be directly converted into serenity's [`Error`](SerenityError).
    #[derive(Debug)]
    pub enum Error {
        /// Error returned by serenity.
        SerenityError(SerenityError),
        /// Error returned when an operation times out.
        TimeoutError,
        /// Error returned when user's choice is invalid.
        InvalidChoice,
        /// Error returned for all other cases.
        Other(String),
    }

    impl StdError for Error {}

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let err = match self {
                Error::SerenityError(e) => Cow::from(e.to_string()),
                Error::TimeoutError => Cow::from("You took too long to respond."),
                Error::InvalidChoice => Cow::from("Invalid choice!"),
                Error::Other(e) => Cow::from(e),
            };

            write!(f, "{}", err)
        }
    }

    impl<'a> From<&'a str> for Error {
        fn from(error: &'a str) -> Self {
            Self::Other(error.to_string())
        }
    }

    impl From<String> for Error {
        fn from(error: String) -> Self {
            Self::Other(error)
        }
    }

    impl From<SerenityError> for Error {
        fn from(error: SerenityError) -> Self {
            Self::SerenityError(error)
        }
    }
}

pub mod misc {
    //! Miscellaneous utility functions to aid with performing common tasks.

    use serenity::{
        model::prelude::{Message, ReactionType},
        prelude::Context,
        Error,
    };

    /// Adds reactions in a non-blocking fashion.
    ///
    /// This allows you to perform other tasks while reactions are being added. This
    /// works by creating a separate task for adding emojis in the background. The
    /// order of `emojis` is preserved.
    ///
    /// See [`add_reactions_blocking`] to add reactions in a blocking fashion. This
    /// function is slightly less efficient than the blocking counterpart.
    ///
    /// [`add_reactions_blocking`]: add_reactions_blocking()
    pub async fn add_reactions(
        ctx: &Context,
        msg: &Message,
        emojis: Vec<ReactionType>,
    ) -> Result<(), Error> {
        let channel_id = msg.channel_id;
        let msg_id = msg.id;
        let http = ctx.http.clone();

        tokio::spawn(async move {
            for emoji in emojis {
                http.create_reaction(channel_id.0, msg_id.0, &emoji).await?;
            }

            Result::<_, Error>::Ok(())
        });

        Ok(())
    }

    /// Adds reactions in a blocking fashion.
    ///
    /// This blocks the execution of code until all reactions are added. The order
    /// of `emojis` is preserved.
    ///
    /// See [`add_reactions`] to add reactions in a non-blocking fashion.
    ///
    /// [`add_reactions`]: add_reactions()
    pub async fn add_reactions_blocking(
        ctx: &Context,
        msg: &Message,
        emojis: &[ReactionType],
    ) -> Result<(), Error> {
        for emoji in emojis {
            ctx.http
                .create_reaction(msg.channel_id.0, msg.id.0, emoji)
                .await?;
        }

        Ok(())
    }
}
