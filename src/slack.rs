//! The Slack hook utilities and encapsulation for sending notification.

use slack_hook::{Payload, PayloadBuilder, Slack};

/// A slack hook encapsulation for sending messages and notifications.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::slack::SlackHookBuilder;
///
/// let hook = SlackHookBuilder::default()
///     .channel("#your-channel")
///     .username("your-bot")
///     .slack_endpoint("https://hooks.slack.com/services/your/slack/endpoint")
///     .expect("failed to construct slack hook")
///     .build()
///     .expect("failed to construct SlackHook");
///
/// println!("{hook:?}");
/// ```
#[derive(Debug, derive_builder::Builder)]
#[builder(setter(into))]
pub struct SlackHook {
    /// The [`Slack`] instance for sending messages.
    #[builder(setter(custom))]
    slack: Slack,

    /// The channel to send to.
    channel: String,
    /// The username of the sender.
    username: String,
}

impl SlackHookBuilder {
    /// Set the endpoint URI of [`SlackHook`].
    pub fn slack_endpoint(&mut self, hook: &str) -> Result<&mut Self, slack_hook::Error> {
        self.slack = Some(Slack::new(hook)?);
        Ok(self)
    }
}

impl SlackHook {
    /// Send the specified message text to the slack channel.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::slack::SlackHookBuilder;
    ///
    /// let hook = SlackHookBuilder::default()
    ///     .channel("#your-channel")
    ///     .username("your-bot")
    ///     .slack_endpoint("https://hooks.slack.com/services/your/slack/endpoint")
    ///     .expect("failed to construct slack hook")
    ///     .build()
    ///     .expect("failed to construct SlackHook");
    ///
    /// // Should be false since I mocked the 'slack_endpoint'.
    /// assert!(!hook.send("hello, world!").is_ok());
    /// ```
    pub fn send(&self, text: &str) -> SlackResult<()> {
        let payload = self.build_send_payload(text)?;

        self.slack
            .send(&payload)
            .map_err(SlackError::SendMessageFailed)
    }

    /// Build the payload to send.
    fn build_send_payload(&self, text: &str) -> SlackResult<Payload> {
        PayloadBuilder::new()
            .text(text)
            .channel(&self.channel)
            .username(&self.username)
            .build()
            .map_err(SlackError::ConstructPayloadFailed)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SlackError {
    #[error("failed to construct payload: {0}")]
    ConstructPayloadFailed(slack_hook::Error),

    #[error("failed to send message: {0}")]
    SendMessageFailed(slack_hook::Error),
}

pub type SlackResult<T> = Result<T, SlackError>;
