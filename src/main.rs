use octocrab::models;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const BASE_PATH: &str = "asset";
const GENERATE: &str = "asset/generate_info.json";

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    handle_issues().await?;

    Ok(())
}

struct GenerateArgs {
    token: String,
    repo_name: String,
    issue_number: Option<String>,
}

fn get_args() -> GenerateArgs {
    let token = env::args().nth(1).expect("missing token");
    let repo_name = env::args().nth(2).expect("missing repo name");
    let issue_number = env::args().nth(3);

    return GenerateArgs {
        token,
        repo_name,
        issue_number,
    };
}

fn init_octocrab(token: String) -> octocrab::Result<Octocrab> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LabelAlter {
    id: models::LabelId,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssueAlter {
    id: models::IssueId,
    title: String,
    number: i64,
    labels: Vec<LabelAlter>,
}

async fn handle_issues() -> octocrab::Result<()> {
    let GenerateArgs {
        token,
        repo_name,
        issue_number,
    } = get_args();
    let octocrab = init_octocrab(token)?;

    let user = get_user(&octocrab).await?;

    let issues = octocrab
        .issues(&user.login, &repo_name)
        .list()
        .creator(&user.login)
        .send()
        .await?;

    let issues_alter: Vec<IssueAlter> = issues
        .clone()
        .into_iter()
        .map(|issue| IssueAlter {
            id: issue.id,
            title: issue.title,
            number: issue.number,
            labels: issue
                .labels
                .iter()
                .map(|label| LabelAlter {
                    id: label.id,
                    name: label.name.to_string(),
                })
                .collect(),
        })
        .collect();

    // 获取要生成 md 的 issue
    let current_generate_info = get_generate_issues();

    let current_generate_info_map: HashSet<models::IssueId> = current_generate_info
        .clone()
        .into_iter()
        .map(|issue| issue.id)
        .collect();

    let issues_to_generate: Vec<models::issues::Issue> = if current_generate_info.len() == 0 {
        issues.clone().items
    } else {
        issues
            .clone()
            .into_iter()
            .filter(|issue| {
                !current_generate_info_map.contains(&issue.id)
                    || (issue_number.is_some()
                        && issue.id.into_inner()
                            == issue_number.as_ref().unwrap().parse::<u64>().unwrap())
            })
            .collect()
    };

    println!("{:#?}", issues_alter);

    // 写入新的 issues 信息
    let new_generate_info_json = serde_json::to_string(&issues_alter).unwrap();

    let mut file = get_overwrite_file(GENERATE);

    match file.write(&new_generate_info_json.as_bytes()) {
        Err(why) => println!("fail to write new issues info: {}", why),
        Ok(_) => println!("write new issues info success!"),
    };

    // 生成md
    for issue in issues_to_generate {
        let mut file = get_overwrite_file(Path::new(BASE_PATH).join(issue.title + ".md"));

        file.write(issue.body.unwrap_or_default().as_bytes())
            .unwrap();
    }

    Ok(())
}

fn get_overwrite_file<P: AsRef<Path>>(path: P) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap()
}

async fn get_user(octocrab: &Octocrab) -> octocrab::Result<models::User> {
    octocrab.current().user().await
}

fn get_generate_issues() -> Vec<IssueAlter> {
    let path = Path::new(GENERATE);
    let mut generate_info = String::new();

    fs::create_dir_all(BASE_PATH).unwrap();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .unwrap();

    match file.read_to_string(&mut generate_info) {
        Err(why) => panic!("couldn't open {}: {}", path.to_string_lossy(), why),
        Ok(_) => println!("read {} successfully", path.to_string_lossy()),
    }

    return serde_json::from_str::<Vec<IssueAlter>>(&generate_info).unwrap_or_default();
}
