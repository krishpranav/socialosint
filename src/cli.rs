use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "socialosint")]
#[command(about = "Social OSINT - Email gathering from Instagram, LinkedIn, and Twitter", long_about = None)]
pub struct Args {
    #[arg(long, required = true, help = "Credentials in a JSON file")]
    pub credentials: PathBuf,

    #[arg(long, help = "Save users, users ID and emails found in a file")]
    pub output: Option<PathBuf>,

    #[arg(long, help = "Searches all the emails published by users in PwnDB")]
    pub pwndb: bool,

    #[arg(long, default_value = "127.0.0.1:9050", help = "Set Tor proxy")]
    pub tor_proxy: String,

    #[arg(long, help = "Use Instagram functions")]
    pub instagram: bool,

    #[arg(long, help = "Get information about locations and their IDs")]
    pub info: Option<String>,

    #[arg(long, help = "Get users with public email from a location ID")]
    pub location: Option<String>,

    #[arg(long, help = "Get users with public email from a hashtag")]
    pub hashtag_ig: Option<String>,

    #[arg(
        long,
        help = "Get email, user ID, followers and followings of a specific username"
    )]
    pub target_ig: Option<String>,

    #[arg(long, help = "Search any user in Instagram")]
    pub search_users_ig: Option<String>,

    #[arg(long, help = "Get users with public email from your followers")]
    pub my_followers: bool,

    #[arg(long, help = "Get users with public email from your followings")]
    pub my_followings: bool,

    #[arg(
        long,
        help = "Get users with public emails from the followers of a target"
    )]
    pub followers_ig: bool,

    #[arg(
        long,
        help = "Get users with public emails from the followings of a target"
    )]
    pub followings_ig: bool,

    #[arg(long, help = "Use LinkedIn functions")]
    pub linkedin: bool,

    #[arg(
        long,
        help = "Get information about a specific company from company ID"
    )]
    pub company: Option<String>,

    #[arg(long, help = "Search any company")]
    pub search_companies: Option<String>,

    #[arg(long, help = "Get the employees of a company and contact information")]
    pub employees: bool,

    #[arg(long, help = "Display my contacts and their contact information")]
    pub my_contacts: bool,

    #[arg(long, help = "Display contacts from a specific user ID")]
    pub user_contacts: Option<String>,

    #[arg(long, help = "Search any user in LinkedIn")]
    pub search_users_in: Option<String>,

    #[arg(long, help = "Get a user's contact information")]
    pub target_in: Option<String>,

    #[arg(long, help = "Send contact request for all users")]
    pub add_contacts: bool,

    #[arg(long, help = "Send contact request for a single user with his user ID")]
    pub add_a_contact: Option<String>,

    #[arg(long, help = "Use Twitter functions")]
    pub twitter: bool,

    #[arg(long, default_value = "100", help = "Number of Tweets to pull")]
    pub limit: usize,

    #[arg(long, help = "Filter Tweets before specified year")]
    pub year: Option<i32>,

    #[arg(long, help = "Filter Tweets sent since date (Example: 2017-12-27)")]
    pub since: Option<String>,

    #[arg(long, help = "Filter Tweets sent until date (Example: 2017-12-27)")]
    pub until: Option<String>,

    #[arg(
        long,
        help = "Slow, but effective method of collecting a user's Tweets and RT"
    )]
    pub profile_full: bool,

    #[arg(long, help = "Search all Tweets associated with a user")]
    pub all_tw: bool,

    #[arg(long, help = "User's Tweets you want to scrape")]
    pub target_tw: Option<String>,

    #[arg(long, help = "Get tweets containing emails from a hashtag")]
    pub hashtag_tw: Option<String>,

    #[arg(long, help = "Scrape a person's followers")]
    pub followers_tw: bool,

    #[arg(long, help = "Scrape a person's follows")]
    pub followings_tw: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
