use slack_hook::{Slack, PayloadBuilder};

pub fn slack_send(text: &str) -> Result<(), String>  {
    let slack = Slack::new(
        "https://hooks.slack.com/services/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",
    )
    .unwrap();
    let p = PayloadBuilder::new()
        .text(text)
        .channel("#wmjtyd-stock")
        .username("My Bot")
        .build()
        .unwrap();

    let res = slack.send(&p);
    match res {
        Ok(()) => Ok(()),
        Err(x) => Err(x.to_string())
    }
}