use json::object;
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
                    let browser2;
                    if browser == "chrome" {
                        browser2 = Browser::Chrome;
                    } else {
                        browser2 = Browser::Firefox;
                    }
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

                // Launch
                launch_bot(&username, &password, hashtags, browser);
            }
            3..=4 => println!("En cours de développement."),
            _ => println!("Inconnu."),
        }
    }
}

fn launch_bot(username: &str, password: &str, hashtags: Vec<String>, browser: Browser) {
    let mut likes = 0;
    let mut limit = 9999;
    let mut post_processed = 0;

    let session = Session::new(browser).expect("Echec de création de la session");
    let mut tab = session.get_selected_tab().unwrap();
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

    for hashtag in hashtags {
        let mut url = String::from("https://www.instagram.com/explore/tags/");
        url += &hashtag;
        url.push_str("/");
        tab.navigate(&url).unwrap();
        thread::sleep(Duration::from_millis(2000));

        while post_processed < limit {
            let x = (post_processed % 3) + 1;
            let y = ((post_processed - (x - 1)) / 3) + 1;
            post_processed += 1;

            let mut xpath = String::from("/html/body/span/section/main/article/div[2]/div/div[");
            if y > 12 {
                xpath += "13";
            } else {
                xpath += &y.to_string();
            }
            xpath.push_str("]/div[");
            xpath += &x.to_string();
            xpath.push_str("]/a");

            // /html/body/div[3]/div[2]/div/article/header/div[2]/div[1]/div[1]/h2/a

            if let Ok(result) = tab.find(Selector::XPath, &xpath) {
                if let Some(mut image) = result {
                    if let Ok(()) = image.click() {
                        thread::sleep(Duration::from_millis(4000));
                    } else {
                        eprintln!("Can't click image.");
                        continue;
                    }
                } else {
                    eprintln!("Can't find image.");
                    continue;
                }
            } else {
                eprintln!("Can't search image.");
                continue;
            }
            

            if let Ok(result) = tab.find(Selector::XPath, "/html/body/div[3]/div[2]/div/article/div[2]/section[1]/span[1]/button/span") {
                if let Some(mut heart) = result {
                    if let Ok(()) = heart.click() {
                        thread::sleep(Duration::from_millis(1000));
                        likes += 1;
                    } else {
                        eprintln!("Can't click the heart.",);
                    }
                } else {
                    eprintln!("Can't find the heart.");
                }
            } else {
                eprintln!("Can't search the heart.");
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
                continue;
            }
        }
    }
}
