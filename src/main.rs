use json::{
    object,
    array,
    JsonValue
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
use webdriver::tab::*;
use std::str::FromStr;
use colored::*;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LIKES: usize = 40;
const SECONDS_BEFORE_RELIKING: usize = 86400*2;

struct User {
    username: String,
    last_like_timestamp: u64,
}

impl User {
    pub fn new(username: String) -> Self {
        let start = SystemTime::now();
        User {
            username,
            last_like_timestamp: start.duration_since(UNIX_EPOCH).expect("Error: failed to read time").as_secs()
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl PartialEq<String> for User {
    fn eq(&self, other: &String) -> bool {
        &self.username == other
    }
}

fn configurate() {
    let mut username = String::new();
    let mut password = String::new();
    let mut hashtags = String::new();
    let mut browser = String::new();

    println!("Bienvenue dans l'outil d'automatisation de likes.");
    println!(
        "Le bot se connectera à votre compte et likera des publications dans des hashtags ciblés."
    );
    println!("Vous allez faire une configuration. Elle sera enregistrée mais modifiable.");
    println!("Quel est votre nom d'utilisateur ?");
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    println!("Quel est votre mot de passe ? Il sera stocké localement mais pas crypté. Pour plus de sécurité vous pouvez opter pour l'option de le renseigner à chaque lancement du bot en écrivant \"secret\" (sans les guillemets).");
    io::stdin()
        .read_line(&mut password)
        .expect("Failed to read line");
    println!("Donnez la liste des hashtags que vous voulez cibler, séparés par des espaces et non précédés par le '#'.");
    io::stdin()
        .read_line(&mut hashtags)
        .expect("Failed to read line");
    println!("Répondre \"oui\" pour utiliser Firefox.");
    io::stdin()
        .read_line(&mut browser)
        .expect("Failed to read line");

    if browser == "oui\n" {
        browser = String::from("firefox");
    } else {
        browser = String::from("chrome");
    }

    username = username.trim().to_string();
    password = password.trim().to_string();
    hashtags = hashtags.trim().to_string();
    let hashtags: Vec<&str> = hashtags.split(" ").collect();

    let mut config_file =
        File::create("config.txt").expect("Impossible de créer le fichier config.txt.");
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
        .expect("Impossible d'écrire dans le fichier config.txt.");
    config_file.sync_all();

    println!("Configuration terminée et enregistrée.");
}

fn read_config() -> Result<(String, String, Vec<String>, Browser), ()> {
    let file = File::open("config.txt");
    if let Err(_) = file {
        println!("Can't open config.txt.");
        return Err(());
    }
    let file = file.unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents);
    let contents = json::parse(&contents);
    if let Err(_) = contents {
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
    let mut config = read_config();
    while config.is_err() {
        println!("Une configuration doit être effectuée.");
        configurate();
        config = read_config();
    }
    let (username, mut password, hashtags, browser) = config.unwrap();
    loop {
        println!("Choisissez une action :");
        println!("[1] : Reconfigurer");
        println!("[2] : Lancer le bot sur un hashtag donné");
        println!(
            "[3] : Lancer le bot sur tous les hashtags rensignés dans le fichier de configuration"
        );
        println!("[4] : Voir les stats");

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
                println!("Redémmarage nécessaire.");
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
                let likes: u32 = likes.trim().to_string().parse().unwrap();

                // Launch
                launch_bot(&username, &password, hashtags, browser, likes);
            }
            3..=4 => println!("Please wait beta version to use that."),
            _ => println!("Inconnu."),
        }
    }
}

impl From<User> for JsonValue {
    fn from(val: User) -> JsonValue {
        object!{
            "username" => val.username,
            "timestamp" => val.last_like_timestamp
        }
    }
}

fn save_usernames(usernames: Vec<(User)>) {
    if let Ok(mut file) = File::create("targets.json") {
        if let Err(e) = file.write_all(json::stringify(json::object!(
            "likes" => usernames,
        )).as_bytes()) {
            eprintln!("Failed to write data in data.json ({}).", e);
        }
    } else {
        eprintln!("Failed to open or create data.json.");
    }
}

fn read_usernames() -> Vec<User> {
    if let Ok(file) = std::fs::read_to_string("targets.json") {
        if let Ok(json) = json::parse(&file) {
            let mut users: Vec<User> = Vec::new();
            let mut i = 0;
            while !json["likes"][i].is_null() {
                if json["likes"][i]["username"].is_string() {
                    if let Some(timestamp) = json["likes"][i]["timestamp"].as_u64() {
                        users.push(
                            User {
                                username: json["likes"][i]["username"].to_string(),
                                last_like_timestamp: timestamp,
                            }
                        )
                    } else {
                        eprintln!("Error 4 when trying to read targets.json.");
                    }
                } else {
                    eprintln!("Error 3 when trying to read targets.json.");
                }
                i+=1;
            }

            users
        } else {
            eprintln!("Error 2 when trying to read targets.json.");
            Vec::new()
        }
        
    } else {
        eprintln!("Error 1 when trying to read targets.json.");
        Vec::new()
    }
}

fn user_must_be_ignored(u1: &User, us: &Vec<User>) -> bool {
    for u2 in us {
        if u1.username == u2.username {
            if u1.last_like_timestamp - SECONDS_BEFORE_RELIKING as u64 >= u2.last_like_timestamp {
                return false;
            } else {
                return true;
            }
        } else {
            return false;
        }
    }
    false
}

#[allow(clippy::cognitive_complexity)]
fn launch_bot(username: &str, password: &str, hashtags: Vec<String>, browser: Browser, likes_limit: u32) {
    let mut likes = 0;
    let mut post_processed = 0;
    let mut errors = 0;

    let session = if let Ok(result) = Session::new(browser) {
        result
    } else {
        eprintln!("Can't launch webdriver session. Please verify that selenium is running. Maybe you need to replace the driver of your browser to match the version of it.");
        return;
    };
    let mut tab = if let Ok(result) = session.get_selected_tab() {
        result
    } else {
        eprintln!("Can't select tab.");
        return;
    };

    tab.navigate("https://www.instagram.com/accounts/login/?source=auth_switcher")
        .unwrap();
    thread::sleep(Duration::from_millis(5000));
    let mut username_block = tab
        .find(Selector::Css, "input[name=\"username\"]")
        .unwrap()
        .unwrap();
    let mut password_block = tab
        .find(Selector::Css, "input[name=\"password\"]")
        .unwrap()
        .unwrap();
    let mut submit_block = tab
        .find(Selector::Css, "button[type=\"submit\"]")
        .unwrap()
        .unwrap();
    username_block.type_text(username).unwrap();
    password_block.type_text(password).unwrap();
    submit_block.click().unwrap();

    while let None = tab
        .find(
            Selector::XPath,
            "/html/body/div[3]/div/div/div[3]/button[2]",
        )
        .unwrap()
    {
        thread::sleep(Duration::from_millis(100));
    }

    let mut notif = tab
        .find(
            Selector::XPath,
            "/html/body/div[3]/div/div/div[3]/button[2]",
        )
        .unwrap()
        .unwrap();
    notif.click().unwrap();

    let mut users: Vec<User> = read_usernames();
    for hashtag in hashtags {
        let mut url = String::from("https://www.instagram.com/explore/tags/");
        url += &hashtag;
        url.push_str("/");
        tab.navigate(&url).unwrap();
        thread::sleep(Duration::from_millis(2000));

        while likes < likes_limit && errors <= 10 {
            let x = (post_processed % 3) + 1;
            let mut y = ((post_processed - (x - 1)) / 3) + 1;
            post_processed += 1;

            let mut xpath = String::from("/html/body/span/section/main/article/div[2]/div/div[");
            if y > 11 {
                if x == 1 {
                    y = 12;
                } else {
                    y = 11;
                }
            }
            xpath += &y.to_string();
            xpath.push_str("]/div[");
            xpath += &x.to_string();
            xpath.push_str("]/a");

            if let Ok(result) = tab.find(Selector::XPath, &xpath) {
                if let Some(mut image) = result {
                    if let Ok(()) = image.click() {
                        thread::sleep(Duration::from_millis(4000));
                    } else {
                        eprintln!("Can't click image.");
                        errors += 1;
                        continue;
                    }
                } else {
                    eprintln!("Can't find image (xpath: {}).", xpath);
                    errors += 1;
                    continue;
                }
            } else {
                eprintln!("Can't search image.");
                errors += 1;
                continue;
            }

            let user: User;
            if let Ok(result) = tab.find(Selector::XPath, "/html/body/div[3]/div[2]/div/article/header/div[2]/div[1]/div[1]/h2/a") {
                if let Some(username_element) = result {
                    if let Ok(value) = username_element.get_text() {
                        user = User::new(value);
                    } else {
                        eprintln!("Can't read username.");
                        errors += 1;
                        continue;
                    }
                } else {
                    eprintln!("Can't find username.");
                    errors += 1;
                    continue;
                }
            } else {
                eprintln!("Can't search username.");
                errors += 1;
                continue;
            }

            let mut total_likes: usize = 0;
            if let Ok(result) = tab.find(Selector::XPath, "/html/body/div[3]/div[2]/div/article/div[2]/section[2]/div/div/button") {
                if let Some(likes_counter) = result {
                    if let Ok(value) = likes_counter.get_text() {
                        let value: Vec<&str> = value.split(' ').collect();
                        if let Ok(number) = FromStr::from_str(value[0]) {
                            total_likes = number;
                        }
                    } else {
                        eprintln!("Can't read the likes counter.",);
                    }
                } else {
                    eprintln!("Can't find the likes counter.");
                }
            } else {
                eprintln!("Can't search the likes counter.");
                errors += 1;
                continue;
            }

            if let Ok(result) = tab.find(Selector::XPath, "/html/body/div[3]/div[2]/div/article/div[2]/section[1]/span[1]/button/span") {
                if let Some(mut heart) = result {
                    if user_must_be_ignored(&user, &users) {
                        println!("{} has already received a like.", user.username);
                    } else if total_likes <= MAX_LIKES {
                        if let Ok(()) = heart.click() {
                            thread::sleep(Duration::from_millis(1000));
                            users.push(user);
                            likes += 1;
                        } else {
                            eprintln!("Can't click the heart.",);
                        }
                    } else {
                        println!("Too much likes ({}/{}) ! Ignoring this post...", total_likes, MAX_LIKES);
                    }
                } else {
                    eprintln!("Can't find the heart.");
                }
            } else {
                eprintln!("Can't search the heart.");
                errors += 1;
                continue;
            }
            

            if let Ok(result) = tab.find(Selector::XPath, "/html/body/div[3]/button[1]") {
                if let Some(mut close) = result {
                    if let Ok(()) = close.click() {
                        thread::sleep(Duration::from_millis(1500));
                    } else {
                        eprintln!("Can't click close button.",);
                    }
                } else {
                    eprintln!("Can't find close button.");
                }
            } else {
                eprintln!("Can't search close button.");
                errors += 1;
                continue;
            }

            if errors > 0 {
                errors -= 1;
            }
        }

        if errors > 10 {
            println!("Too much errors. Stopping process.");
        }
    }

    save_usernames(users);
}
