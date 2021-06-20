use lazy_regex::regex;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::id::ChannelId;

// TODO: use `parse_channel` https://docs.rs/serenity/0.10.5/serenity/utils/fn.parse_channel.html
pub fn channel_mention(msg: &str) -> Vec<ChannelId> {
    // rust-analyzer intellisense
    let regex: &Regex = regex!(r"<#(\d+)>");

    regex
        .captures_iter(msg)
        .map(|captures| captures.get(1).unwrap())
        .filter_map(|m| m.as_str().parse::<u64>().ok())
        .map(|number| ChannelId(number))
        .collect()
}
