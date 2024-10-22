use json::{
    object
};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::str;
use std::thread;
use std::time::Duration;
use webdriver::enums::*;
use webdriver::session::*;

fn configurate() {
    let mut username = String::new();
    let mut password = String::new();
    let mut hashtags = String::new();
    let mut browser = String::new();

    println!("Welcome to this instagram bot.");
    println!(
        "The bot will connect on your account and will likes publications in targeted hashtags."
    );
    println!("You need to configurate the bot. Configuration will be saved and you will be able to update informations.");
    println!("What is your instagram username ?");
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    println!("What is your password ? He will be stored in config.txt and readable by everyone. You can type \"secret\" if you don't want to store your password.");
    io::stdin()
        .read_line(&mut password)
        .expect("Failed to read line");
    println!("Write the list of hashtags you want to target, separed by spaces and without the caracter '#'.");
    io::stdin()
        .read_line(&mut hashtags)
        .expect("Failed to read line");
    println!("Say \"great!\" if you use Firefox. (otherwise the bot will use chrome)");
    io::stdin()
        .read_line(&mut browser)
        .expect("Failed to read line");
    
    if browser.trim() == "great!" {
        browser = String::from("firefox");
    } else {
        browser = String::from("chrome");
    }

    username = username.trim().to_string();
    password = password.trim().to_string();
    hashtags = hashtags.trim().to_string();
    let hashtags: Vec<&str> = hashtags.split(' ').collect();

    let mut config_file =
        File::create("config.txt").expect("Unable to write in file config.txt.");
    config_file
        .write_all(
            json::stringify(json::object!(
                "username" => username,
                "password" => password,
                "hashtags" => hashtags,
                "browser" => browser
            ))
            .as_bytes(),
        )
        .expect("Unable to write in file config.txt.");

    config_file.sync_all();

    println!("Configuration done!");
}

fn read_config() -> Result<(String, String, Vec<String>, Browser), ()> {
    let file = File::open("config.txt");
    if file.is_err() {
        println!("Can't open config.txt.");
        return Err(());
    }
    let file = file.unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    if buf_reader.read_to_string(&mut contents).is_err() {
        println!("Can't read config.txt.");
        return Err(());
    }
    let contents = json::parse(&contents);
    if contents.is_err() {
        println!("config.txt contains invalid data.");
        return Err(());
    }
    let contents = contents.unwrap();

    if let Some(username) = contents["username"].as_str() {
        if let Some(password) = contents["password"].as_str() {
            if contents["hashtags"] != json::JsonValue::Null {
                let mut hashtags: Vec<String> = Vec::new();
                let mut i = 0;
                while let Some(hashtag) = contents["password"][i].as_str() {
                    hashtags.push(hashtag.to_string());
                    i += 1;
                }
                if let Some(browser) = contents["browser"].as_str() {
                    let browser2 = if browser == "chrome" { Browser::Chrome } else { Browser::Firefox };
                    return Ok((
                        username.to_string(),
                        password.to_string(),
                        hashtags,
                        browser2,
                    ));
                } else {
                    println!("Missing field browser in config.txt.");
                }
            } else {
                println!("Missing field hashtags in config.txt.");
            }
        } else {
            println!("Missing field password in config.txt.");
        }
    } else {
        println!("Missing field username in config.txt.");
    }

    Err(())
}

fn main() {
    if cfg!(debug_assertions) {
        env_logger::init();
    }
    
    let mut config = read_config();
    while config.is_err() {
        println!("You need to do a configuration.");
        configurate();
        config = read_config();
    }
    let (username, mut password, _hashtags, browser) = config.unwrap();
    loop {
        println!("Choose an action number :");
        println!("[1] : Do configuration again");
        println!("[2] : Launch but on a hashtag you will choose");
        println!(
            "[3] : Launch bot on every hashtags stored in config.txt file."
        );
        println!("[4] : See stats");

        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("Failed to read line");

        let trimmed = input_text.trim();
        let answer = match trimmed.parse::<u32>() {
            Ok(i) => i,
            Err(_) => continue,
        };

        match answer {
            1 => {
                configurate();
                println!("YOU MUST LAUNCH THIS PROGRAM AGAIN.");
            }
            2 => {
                // Read the hashtag
                let mut hashtags: Vec<String> = Vec::new();
                let mut hashtag = String::new();
                println!("Enter the hashtag");
                io::stdin()
                    .read_line(&mut hashtag)
                    .expect("Failed to read line");
                hashtag = hashtag.trim().to_string();
                hashtags.push(hashtag);
                
                // Read the password
                if password == "secret" {
                    println!("Enter your password");
                    password = String::new();
                    io::stdin()
                        .read_line(&mut password)
                        .expect("Failed to read line");
                    password = password.trim().to_string();
                }

                // Read the number likes to do
                println!("How many likes to do?");
                let mut likes = String::new();
                io::stdin()
                    .read_line(&mut likes)
                    .expect("Failed to read line");
                let likes: usize = likes.trim().to_string().parse().unwrap();

                // Launch
                launch_bot(&username, &password, hashtags, browser, likes);
            }
            3..=4 => println!("Please wait beta version to use that."),
            _ => println!("Inconnu."),
        }
    }
}

#[allow(clippy::cognitive_complexity)]
fn launch_bot(username: &str, password: &str, hashtags: Vec<String>, browser: Browser, likes_limit: usize) {
    let likes_limit = likes_limit / hashtags.len();
    let session = Session::new(browser).expect("Failed to create session.");

    let mut tab = session.get_selected_tab().expect("Failed to get tab");
    tab.navigate("https://www.instagram.com/accounts/login/?source=auth_switcher").expect("Failed to load page");
    thread::sleep(Duration::from_secs(5));
    if let Ok(input) = tab.find(Selector::XPath, "/html/body/span/section/main/div/article/div/div[1]/div/form/div[2]/div/label/input") {
        if let Some(mut input) = input {
            if input.type_text(username).is_err() {
                eprintln!("Error while sending text to input");
                return;
            }
        } else {
            eprintln!("Can't find input");
            return;
        }
    } else {
        eprintln!("Can't search input");
        return;
    }
    if let Ok(input) = tab.find(Selector::XPath, "/html/body/span/section/main/div/article/div/div[1]/div/form/div[3]/div/label/input") {
        if let Some(mut input) = input {
            if input.type_text(password).is_err() {
                eprintln!("Error while sending text to password input");
                return;
            }
        } else {
            eprintln!("Can't find password input");
            return;
        }
    } else {
        eprintln!("Can't search password input");
        return;
    }
    if let Ok(input) = tab.find(Selector::XPath, "/html/body/span/section/main/div/article/div/div[1]/div/form/div[4]/button") {
        if let Some(mut input) = input {
            if input.click().is_err() {
                eprintln!("Error while clicking submit button");
                return;
            }
        } else {
            eprintln!("Can't find submit button");
            return;
        }
    } else {
        eprintln!("Can't search submit button");
        return;
    }
    thread::sleep(Duration::from_secs(5));
    if let Ok(url) = tab.get_url() {
        if url == "https://www.instagram.com/accounts/login/?source=auth_switcher" {
            if let Ok(message) = tab.find(Selector::XPath, "//*[@id=\"slfErrorAlert\"]") {
                if let Some(message) = message {
                    if let Ok(message) = message.get_text() {
                        eprintln!("{}", message);
                        return;
                    } else {
                        eprintln!("Can't read error message");
                        return;
                    }
                } else {
                    eprintln!("Can't find error message");
                    return;
                }
            } else {
                eprintln!("Can't search error messages");
                return;
            }
        }
    } else {
        eprintln!("Can't get url");
        return;
    }
    println!("The bot is connected!");

    for hashtag in hashtags {
        if tab.navigate(&format!("https://www.instagram.com/explore/tags/{}/?hl=en", hashtag)).is_err() {
            eprintln!("Can't navigate to {} hashtag", hashtag);
            return;
        }

        thread::sleep(Duration::from_secs(10));

        if let Ok(post) = tab.find(Selector::XPath, "/html/body/span/section/main/article/div[1]/div/div/div[3]/div[3]/a") {
            if let Some(mut post) = post {
                if post.click().is_err() {
                    eprintln!("Error while clicking post");
                    return;
                }
            } else {
                eprintln!("Can't find post");
                return;
            }
        } else {
            eprintln!("Can't search post");
            return;
        }

        thread::sleep(Duration::from_secs(10));

        let mut post_liked = 0;
        while post_liked < likes_limit {
            if let Ok(next_button) = tab.find(Selector::XPath, "/html/body/div[3]/div[1]/div/div/a[2]") {
                if let Some(mut next_button) = next_button {
                    if next_button.click().is_err() {
                        eprintln!("Error while clicking \"next\" button");
                        continue;
                    }
                } else {
                    eprintln!("Can't find \"next\" button");
                    continue;
                }
            } else {
                eprintln!("Can't search \"next\" button");
                continue;
            }

            thread::sleep(Duration::from_secs(5));

            if let Ok(heart) = tab.find(Selector::XPath, "/html/body/div[3]/div[2]/div/article/div[2]/section[1]/span[1]/button") {
                if let Some(mut heart) = heart {
                    if heart.click().is_err() {
                        eprintln!("Error while clicking heart");
                        continue;
                    } else {
                        post_liked += 1;
                    }
                } else {
                    eprintln!("Can't find heart");
                    continue;
                }
            } else {
                eprintln!("Can't search heart");
                continue;
            }

            thread::sleep(Duration::from_millis(500));

            if let Ok(action_blocked) = tab.find(Selector::XPath, "/html/body/div[4]/div/div/div[2]/button[1]") {
                if let Some(mut action_blocked) = action_blocked {
                    println!("Instagram has detected the bot.");
                    if action_blocked.click().is_ok() {
                        println!("Accusation has been denied.");
                    }
                    println!("Stopping liking process.");
                } else {
                    // Ok
                }
            } else {
                eprintln!("Can't search element");
                continue;
            }

            thread::sleep(Duration::from_millis(600));
        }
    }
}
