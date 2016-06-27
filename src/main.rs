extern crate telegram_bot;
extern crate regex;

use telegram_bot::types::ParseMode::Markdown;
use telegram_bot::*;

use std::process::Command;
use regex::Regex;

/* Git repository info*/
struct GitRepo {
  default_branch: String,  
  remote: String,
  name: String,
  path: String
}

/* Git commit message data */
struct GitCommitMessage {
  message: String,
  author: String,
  date: String,
  id: String,
}

/* methods for commit message (formatting text for response) */
impl GitCommitMessage {
  fn get_text(&self) -> String {
    format!("\nCommit: {}\nAuthor: {}\nDate: {}\nMessage: {}\n", self.id, self.author, self.date, self.message)
  }
}

impl GitRepo {
  /* method for run command for get commits as string schema */
  fn get_logs_schema(&self, branch: String, count: u32) -> String {
    let output_format_json = "--pretty=format:%H|%an <%ae>|%ad|%f".to_string();
    let max_count = format!("--max-count={}", &count).to_string();
    let git_dir = format!("--git-dir={}", &self.path).to_string();

    let exec = format!("git {} log {} {} {}", &git_dir, &branch, &output_format_json, &max_count);
    println!("{}", exec);
   
    /* Matching valid JSON from command output */ 
    let schema_log = match Command::new("git").arg(&git_dir).arg("log").arg(&branch).arg(&output_format_json).arg(max_count)
    
    .output() {
      Ok(out) => format!("{}", String::from_utf8_lossy(&out.stdout)),
      Err(e) => format!("")
    };

    schema_log
  }

  /* parse string schema and return array of commits */
  fn get_commits(&self, schema_data: &String) -> Vec<GitCommitMessage> {
    let commits:Vec<&str> = schema_data.split("\n").collect();
    let mut result:Vec<GitCommitMessage> = vec![];

    for commit in commits {
      let commit_data:Vec<&str> = commit.split("|").collect();
      let commit_object = GitCommitMessage {
        message: commit_data[3].to_string(),
        author: commit_data[1].to_string(),
        date: commit_data[2].to_string(),
        id: commit_data[0].to_string()
      };

      result.push(commit_object);
    }

    result
  }  
}

/* get new logs and send response with logs */
fn get_from_git_and_send_logs(git: &GitRepo, api: &Api, chat_id: i64, branch: String, count: u32) {
  let schema = git.get_logs_schema(branch, count);
  let commits = git.get_commits(&schema);

  let mut message_text_commits: String = "".to_owned();

  for commit in commits {
    let commit_text = commit.get_text().to_owned();
    message_text_commits.push_str(&commit_text); 
  }

  match api.send_message(chat_id, message_text_commits, None, None, None, None) {
    Err(error) => println!("{}", error),
    Ok(msg) => {}
  };
}

/* Handle new text messages */
fn handle_request_message(git: &GitRepo, api: &Api, chat_id: &i64, message: String) {
  let isGetLogs = Regex::new(r"^/get_logs").unwrap().is_match(&message);
  println!("{}", &message);

  if isGetLogs {
    let mut split_group_bot:Vec<&str> = message.split("@").collect();
    let mut msg: String = split_group_bot[0].to_string();

    let chunks: Vec<&str> = msg.split(":").collect();
    let length = chunks.len();

    match length {
      2 => {
        let branch: String = chunks[1].to_string();
        get_from_git_and_send_logs(&git, &api, chat_id.clone(), branch, 10);
      },
      3 => {
        let branch: String = chunks[1].to_string();
        /* Match parse string to u32 integer */
        let count: u32 = match chunks[2].to_string().parse::<u32>() {
          Ok(result) => result,
          Err(e) => 10
        };

        get_from_git_and_send_logs(&git, &api, chat_id.clone(), git.default_branch.clone(), count);
      },
      _ => {
        get_from_git_and_send_logs(&git, &api, chat_id.clone(), git.default_branch.clone(), 10);
      },
    }
  } else {
    let repo_name: String = git.name.clone();
    let help_message: String = format!("I don't understand your command. *Try this matching*:\n```/get_logs@{}_bot_name\n/get_logs:~branch~@{}_bot_name\n/get_logs:~branch~:count@{}_bot_name```",
        repo_name, repo_name, repo_name
    );

    /* If not matching request get_logs - send help message */
    match api.send_message(
      chat_id.clone(), 
      help_message, 
      Some(Markdown), 
      None, None, None
    ) {
      Err(error) => println!("{}", error),
      Ok(msg) => {}
    };    
  }
}

/* Handle unknown message types */
fn handle_none_message(api: &Api) {

}

fn main() {
  let git = GitRepo {
    path: "~your-project/.git".to_string(),    
    default_branch: "master".to_string(),  
    remote: "~remote".to_string(), // Testing TODO
    name: "projectName".to_string(),
  };

  // Create bot, test simple API call and print bot information
  let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();

  // create listener for fetch new data 
  let mut bot = api.listener(ListeningMethod::LongPoll(None));
  println!("Running bot app: {}", git.name);

  bot.listen(|revieve_data| {

    match revieve_data.message {
      Some(responder) => {
        let name = responder.from.first_name;
        let chat_id: i64 = responder.chat.id();

        match responder.msg {
          // If message is text 
          MessageType::Text(text) => handle_request_message(&git, &api, &chat_id, text),
          _ => handle_none_message(&api)
        }
      }

      None => {}
    }

    Ok(ListeningAction::Continue)
  });
}
